#nullable enable
using System;
using System.Collections.Generic;
using System.Reflection;
using DINOForge.SDK.Assets;
using DINOForge.SDK.Models;
using DINOForge.SDK.Universe;
using DINOForge.SDK.Validation;
using DINOForge.SDK.Registry;
using DINOForge.SDK;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Coverage tests for zero-coverage model and utility classes.
    /// Tests simple property setters, constructors, and state management.
    /// </summary>
    [Collection(AssetSwapRegistryCollection.Name)]
    public class ModelCoverageTests
    {
        // Helper to clear AssetSwapRegistry state between tests
        private static void ClearAssetSwapRegistry()
        {
            var clearMethod = typeof(AssetSwapRegistry)
                .GetMethod("Clear", BindingFlags.NonPublic | BindingFlags.Static);
            clearMethod?.Invoke(null, null);
        }

        #region AssetSwapRequest Tests

        [Fact]
        public void AssetSwapRequest_Constructor_SetProperties()
        {
            // Arrange & Act
            var request = new AssetSwapRequest("Assets/Prefabs/Unit.prefab", "/mods/bundle.bundle", "Unit");

            // Assert
            request.AssetAddress.Should().Be("Assets/Prefabs/Unit.prefab");
            request.ModBundlePath.Should().Be("/mods/bundle.bundle");
            request.AssetName.Should().Be("Unit");
            request.Applied.Should().BeFalse();
        }

        [Fact]
        public void AssetSwapRequest_Constructor_ThrowsOnNullAssetAddress()
        {
            // Act & Assert
            var action = () => new AssetSwapRequest(null!, "/mods/bundle.bundle", "Unit");
            action.Should().Throw<ArgumentNullException>().WithParameterName("assetAddress");
        }

        [Fact]
        public void AssetSwapRequest_Constructor_ThrowsOnNullModBundlePath()
        {
            // Act & Assert
            var action = () => new AssetSwapRequest("Assets/Prefabs/Unit.prefab", null!, "Unit");
            action.Should().Throw<ArgumentNullException>().WithParameterName("modBundlePath");
        }

        [Fact]
        public void AssetSwapRequest_Constructor_ThrowsOnNullAssetName()
        {
            // Act & Assert
            var action = () => new AssetSwapRequest("Assets/Prefabs/Unit.prefab", "/mods/bundle.bundle", null!);
            action.Should().Throw<ArgumentNullException>().WithParameterName("assetName");
        }

        [Fact]
        public void AssetSwapRequest_Applied_CanBeSet()
        {
            // Arrange
            var request = new AssetSwapRequest("Assets/Prefabs/Unit.prefab", "/mods/bundle.bundle", "Unit");

            // Act
            request.Applied = true;

            // Assert
            request.Applied.Should().BeTrue();
        }

        #endregion

        #region AssetSwapRegistry Tests

        [Fact]
        public void AssetSwapRegistry_Register_ThenGetPending_ReturnsRequest()
        {
            // Arrange
            ClearAssetSwapRegistry();
            var request = new AssetSwapRequest("Assets/Prefabs/Unit.prefab", "/mods/bundle.bundle", "Unit");

            // Act
            AssetSwapRegistry.Register(request);
            var pending = AssetSwapRegistry.GetPending();

            // Assert
            pending.Should().HaveCount(1);
            pending[0].AssetAddress.Should().Be("Assets/Prefabs/Unit.prefab");

            // Cleanup
            ClearAssetSwapRegistry();
        }

        [Fact]
        public void AssetSwapRegistry_Register_ThrowsOnNull()
        {
            // Arrange
            ClearAssetSwapRegistry();

            // Act & Assert
            var action = () => AssetSwapRegistry.Register(null!);
            action.Should().Throw<ArgumentNullException>();

            // Cleanup
            ClearAssetSwapRegistry();
        }

        [Fact]
        public void AssetSwapRegistry_MarkApplied_RemovesFromPending()
        {
            // Arrange
            ClearAssetSwapRegistry();
            var request = new AssetSwapRequest("Assets/Prefabs/Unit.prefab", "/mods/bundle.bundle", "Unit");
            AssetSwapRegistry.Register(request);

            // Act
            AssetSwapRegistry.MarkApplied("Assets/Prefabs/Unit.prefab");
            var pending = AssetSwapRegistry.GetPending();

            // Assert
            pending.Should().BeEmpty();

            // Cleanup
            ClearAssetSwapRegistry();
        }

        [Fact]
        public void AssetSwapRegistry_MarkApplied_WithEmptyString_DoesNotThrow()
        {
            // Arrange
            ClearAssetSwapRegistry();

            // Act & Assert - should not throw
            var action = () => AssetSwapRegistry.MarkApplied("");
            action.Should().NotThrow();

            // Cleanup
            ClearAssetSwapRegistry();
        }

        [Fact]
        public void AssetSwapRegistry_Count_ReturnsCorrectCount()
        {
            // Arrange
            ClearAssetSwapRegistry();
            var request1 = new AssetSwapRequest("Assets/Prefabs/Unit1.prefab", "/mods/bundle.bundle", "Unit1");
            var request2 = new AssetSwapRequest("Assets/Prefabs/Unit2.prefab", "/mods/bundle.bundle", "Unit2");

            // Act
            AssetSwapRegistry.Register(request1);
            AssetSwapRegistry.Register(request2);
            int count = AssetSwapRegistry.Count;

            // Assert
            count.Should().Be(2);

            // Cleanup
            ClearAssetSwapRegistry();
        }

        #endregion

        #region FactionAudio Tests

        [Fact]
        public void FactionAudio_DefaultConstruction()
        {
            // Act
            var audio = new FactionAudio();

            // Assert
            audio.WeaponPack.Should().BeNull();
            audio.StructurePack.Should().BeNull();
            audio.AmbientPack.Should().BeNull();
            audio.MusicPack.Should().BeNull();
        }

        [Fact]
        public void FactionAudio_SetWeaponPack()
        {
            // Arrange
            var audio = new FactionAudio();

            // Act
            audio.WeaponPack = "weapons_pack_01";

            // Assert
            audio.WeaponPack.Should().Be("weapons_pack_01");
        }

        [Fact]
        public void FactionAudio_SetStructurePack()
        {
            // Arrange
            var audio = new FactionAudio();

            // Act
            audio.StructurePack = "structures_pack_01";

            // Assert
            audio.StructurePack.Should().Be("structures_pack_01");
        }

        [Fact]
        public void FactionAudio_SetAmbientPack()
        {
            // Arrange
            var audio = new FactionAudio();

            // Act
            audio.AmbientPack = "ambient_pack_01";

            // Assert
            audio.AmbientPack.Should().Be("ambient_pack_01");
        }

        [Fact]
        public void FactionAudio_SetAllPacks()
        {
            // Arrange
            var audio = new FactionAudio();

            // Act
            audio.WeaponPack = "weapons_01";
            audio.StructurePack = "structures_01";
            audio.AmbientPack = "ambient_01";
            audio.MusicPack = "music_01";

            // Assert
            audio.WeaponPack.Should().Be("weapons_01");
            audio.StructurePack.Should().Be("structures_01");
            audio.AmbientPack.Should().Be("ambient_01");
            audio.MusicPack.Should().Be("music_01");
        }

        #endregion

        #region FactionVisuals Tests

        [Fact]
        public void FactionVisuals_DefaultConstruction()
        {
            // Act
            var visuals = new FactionVisuals();

            // Assert
            visuals.PrimaryColor.Should().BeNull();
            visuals.AccentColor.Should().BeNull();
            visuals.ProjectilePack.Should().BeNull();
            visuals.UiSkin.Should().BeNull();
        }

        [Fact]
        public void FactionVisuals_SetPrimaryColor()
        {
            // Arrange
            var visuals = new FactionVisuals();

            // Act
            visuals.PrimaryColor = "#FF0000";

            // Assert
            visuals.PrimaryColor.Should().Be("#FF0000");
        }

        [Fact]
        public void FactionVisuals_SetAccentColor()
        {
            // Arrange
            var visuals = new FactionVisuals();

            // Act
            visuals.AccentColor = "#00FF00";

            // Assert
            visuals.AccentColor.Should().Be("#00FF00");
        }

        [Fact]
        public void FactionVisuals_SetBothColors()
        {
            // Arrange
            var visuals = new FactionVisuals();

            // Act
            visuals.PrimaryColor = "#FF0000";
            visuals.AccentColor = "#00FF00";

            // Assert
            visuals.PrimaryColor.Should().Be("#FF0000");
            visuals.AccentColor.Should().Be("#00FF00");
        }

        #endregion

        #region UnitVisuals Tests

        [Fact]
        public void UnitVisuals_DefaultConstruction()
        {
            // Act
            var visuals = new UnitVisuals();

            // Assert
            visuals.Icon.Should().BeNull();
            visuals.Portrait.Should().BeNull();
            visuals.ModelOverride.Should().BeNull();
            visuals.ProjectileVfx.Should().BeNull();
            visuals.MuzzleVfx.Should().BeNull();
        }

        [Fact]
        public void UnitVisuals_SetIcon()
        {
            // Arrange
            var visuals = new UnitVisuals();

            // Act
            visuals.Icon = "Assets/Icons/Unit.png";

            // Assert
            visuals.Icon.Should().Be("Assets/Icons/Unit.png");
        }

        [Fact]
        public void UnitVisuals_SetPortrait()
        {
            // Arrange
            var visuals = new UnitVisuals();

            // Act
            visuals.Portrait = "Assets/Portraits/Unit.png";

            // Assert
            visuals.Portrait.Should().Be("Assets/Portraits/Unit.png");
        }

        [Fact]
        public void UnitVisuals_SetModelOverride()
        {
            // Arrange
            var visuals = new UnitVisuals();

            // Act
            visuals.ModelOverride = "Assets/Models/UnitModel.prefab";

            // Assert
            visuals.ModelOverride.Should().Be("Assets/Models/UnitModel.prefab");
        }

        [Fact]
        public void UnitVisuals_SetAllFields()
        {
            // Arrange
            var visuals = new UnitVisuals();

            // Act
            visuals.Icon = "icon.png";
            visuals.Portrait = "portrait.png";
            visuals.ModelOverride = "model.prefab";
            visuals.ProjectileVfx = "projectile_vfx";
            visuals.MuzzleVfx = "muzzle_vfx";

            // Assert
            visuals.Icon.Should().Be("icon.png");
            visuals.Portrait.Should().Be("portrait.png");
            visuals.ModelOverride.Should().Be("model.prefab");
            visuals.ProjectileVfx.Should().Be("projectile_vfx");
            visuals.MuzzleVfx.Should().Be("muzzle_vfx");
        }

        #endregion

        #region UnitAudio Tests

        [Fact]
        public void UnitAudio_DefaultConstruction()
        {
            // Act
            var audio = new UnitAudio();

            // Assert
            audio.AttackSound.Should().BeNull();
            audio.DeathSound.Should().BeNull();
            audio.SelectSound.Should().BeNull();
            audio.MoveSound.Should().BeNull();
        }

        [Fact]
        public void UnitAudio_SetAttackSound()
        {
            // Arrange
            var audio = new UnitAudio();

            // Act
            audio.AttackSound = "sounds/attack.wav";

            // Assert
            audio.AttackSound.Should().Be("sounds/attack.wav");
        }

        [Fact]
        public void UnitAudio_SetDeathSound()
        {
            // Arrange
            var audio = new UnitAudio();

            // Act
            audio.DeathSound = "sounds/death.wav";

            // Assert
            audio.DeathSound.Should().Be("sounds/death.wav");
        }

        [Fact]
        public void UnitAudio_SetSelectSound()
        {
            // Arrange
            var audio = new UnitAudio();

            // Act
            audio.SelectSound = "sounds/select.wav";

            // Assert
            audio.SelectSound.Should().Be("sounds/select.wav");
        }

        [Fact]
        public void UnitAudio_SetAllSounds()
        {
            // Arrange
            var audio = new UnitAudio();

            // Act
            audio.AttackSound = "attack.wav";
            audio.DeathSound = "death.wav";
            audio.SelectSound = "select.wav";
            audio.MoveSound = "move.wav";

            // Assert
            audio.AttackSound.Should().Be("attack.wav");
            audio.DeathSound.Should().Be("death.wav");
            audio.SelectSound.Should().Be("select.wav");
            audio.MoveSound.Should().Be("move.wav");
        }

        #endregion

        #region GlobalStyle Tests

        [Fact]
        public void GlobalStyle_DefaultConstruction()
        {
            // Act
            var style = new GlobalStyle();

            // Assert
            style.Tone.Should().BeNull();
            style.UiTheme.Should().BeNull();
            style.FontStyle.Should().BeNull();
        }

        [Fact]
        public void GlobalStyle_SetTone()
        {
            // Arrange
            var style = new GlobalStyle();

            // Act
            style.Tone = "gritty";

            // Assert
            style.Tone.Should().Be("gritty");
        }

        [Fact]
        public void GlobalStyle_SetUiTheme()
        {
            // Arrange
            var style = new GlobalStyle();

            // Act
            style.UiTheme = "dark_mode";

            // Assert
            style.UiTheme.Should().Be("dark_mode");
        }

        [Fact]
        public void GlobalStyle_SetFontStyle()
        {
            // Arrange
            var style = new GlobalStyle();

            // Act
            style.FontStyle = "bold_serif";

            // Assert
            style.FontStyle.Should().Be("bold_serif");
        }

        [Fact]
        public void GlobalStyle_SetAllFields()
        {
            // Arrange
            var style = new GlobalStyle();

            // Act
            style.Tone = "clean";
            style.UiTheme = "light_mode";
            style.FontStyle = "sans_serif";

            // Assert
            style.Tone.Should().Be("clean");
            style.UiTheme.Should().Be("light_mode");
            style.FontStyle.Should().Be("sans_serif");
        }

        #endregion

        #region ValidationError Tests

        [Fact]
        public void ValidationError_Constructor_SetsProperties()
        {
            // Act
            var error = new ValidationError("loads.units[0]", "Unit ID must not be empty", "required");

            // Assert
            error.Path.Should().Be("loads.units[0]");
            error.Message.Should().Be("Unit ID must not be empty");
            error.Rule.Should().Be("required");
        }

        [Fact]
        public void ValidationError_Constructor_WithDifferentRule()
        {
            // Act
            var error = new ValidationError("faction.id", "ID must match pattern", "pattern");

            // Assert
            error.Path.Should().Be("faction.id");
            error.Message.Should().Be("ID must match pattern");
            error.Rule.Should().Be("pattern");
        }

        [Fact]
        public void ValidationError_Constructor_WithTypeRule()
        {
            // Act
            var error = new ValidationError("version", "Version must be string", "type");

            // Assert
            error.Rule.Should().Be("type");
        }

        #endregion

        #region PackOverrides Tests

        [Fact]
        public void PackOverrides_DefaultConstruction()
        {
            // Act
            var overrides = new PackOverrides();

            // Assert
            overrides.Units.Should().BeNull();
            overrides.Buildings.Should().BeNull();
            overrides.Stats.Should().BeNull();
        }

        [Fact]
        public void PackOverrides_SetUnits()
        {
            // Arrange
            var overrides = new PackOverrides();

            // Act
            overrides.Units = new List<string> { "overrides/units/unit1.yaml", "overrides/units/unit2.yaml" };

            // Assert
            overrides.Units.Should().HaveCount(2);
            overrides.Units[0].Should().Be("overrides/units/unit1.yaml");
        }

        [Fact]
        public void PackOverrides_SetBuildings()
        {
            // Arrange
            var overrides = new PackOverrides();

            // Act
            overrides.Buildings = new List<string> { "overrides/buildings/barracks.yaml" };

            // Assert
            overrides.Buildings.Should().HaveCount(1);
            overrides.Buildings[0].Should().Be("overrides/buildings/barracks.yaml");
        }

        [Fact]
        public void PackOverrides_SetStats()
        {
            // Arrange
            var overrides = new PackOverrides();

            // Act
            overrides.Stats = new List<string> { "overrides/stats/balance.yaml" };

            // Assert
            overrides.Stats.Should().HaveCount(1);
            overrides.Stats[0].Should().Be("overrides/stats/balance.yaml");
        }

        [Fact]
        public void PackOverrides_SetAllProperties()
        {
            // Arrange
            var overrides = new PackOverrides();
            var unitList = new List<string> { "u1.yaml" };
            var buildingList = new List<string> { "b1.yaml" };
            var statsList = new List<string> { "s1.yaml" };

            // Act
            overrides.Units = unitList;
            overrides.Buildings = buildingList;
            overrides.Stats = statsList;

            // Assert
            overrides.Units.Should().BeSameAs(unitList);
            overrides.Buildings.Should().BeSameAs(buildingList);
            overrides.Stats.Should().BeSameAs(statsList);
        }

        #endregion

        #region RegistryDocument Tests

        [Fact]
        public void RegistryDocument_DefaultConstruction()
        {
            // Act
            var doc = new RegistryDocument();

            // Assert
            doc.Version.Should().Be(string.Empty);
            doc.Updated.Should().Be(string.Empty);
            doc.Packs.Should().BeEmpty();
        }

        [Fact]
        public void RegistryDocument_SetVersion()
        {
            // Arrange
            var doc = new RegistryDocument();

            // Act
            doc.Version = "1.0.0";

            // Assert
            doc.Version.Should().Be("1.0.0");
        }

        [Fact]
        public void RegistryDocument_SetUpdated()
        {
            // Arrange
            var doc = new RegistryDocument();

            // Act
            doc.Updated = "2025-03-11T12:00:00Z";

            // Assert
            doc.Updated.Should().Be("2025-03-11T12:00:00Z");
        }

        [Fact]
        public void RegistryDocument_SetPacks()
        {
            // Arrange
            var doc = new RegistryDocument();
            var packs = new List<RegistryPackEntry>
            {
                new RegistryPackEntry { Id = "pack1", Name = "Pack One" }
            };

            // Act
            doc.Packs = packs;

            // Assert
            doc.Packs.Should().HaveCount(1);
            doc.Packs[0].Id.Should().Be("pack1");
        }

        [Fact]
        public void RegistryDocument_SetAllProperties()
        {
            // Arrange
            var doc = new RegistryDocument();

            // Act
            doc.Version = "2.0.0";
            doc.Updated = "2025-03-11";
            doc.Packs = new List<RegistryPackEntry>
            {
                new RegistryPackEntry { Id = "test-pack", Name = "Test Pack" }
            };

            // Assert
            doc.Version.Should().Be("2.0.0");
            doc.Updated.Should().Be("2025-03-11");
            doc.Packs.Should().HaveCount(1);
        }

        #endregion
    }
}
