#nullable enable
using System.Collections.Generic;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for M13: Total Conversion Framework.
    /// Covers manifest validation, asset replacement engine, and completeness checks.
    /// </summary>
    public class TotalConversionTests
    {
        private static TotalConversionManifest MakeStarWarsManifest() => new()
        {
            Id = "warfare-starwars",
            Name = "Star Wars: The Clone Wars",
            Version = "0.1.0",
            Type = "total_conversion",
            Author = "DINOForge",
            Theme = "star-wars",
            Singleton = true,
            ReplacesVanilla = new Dictionary<string, string>
            {
                ["player"] = "galactic-republic",
                ["enemy_classic"] = "confederacy-of-independent-systems",
                ["enemy_guerrilla"] = "separatist-extremists"
            },
            Factions = new List<TcFactionEntry>
            {
                new() { Id = "galactic-republic", Replaces = "player", Name = "Galactic Republic", Archetype = "order", Units = new() { "clone-trooper", "arc-trooper" } },
                new() { Id = "confederacy-of-independent-systems", Replaces = "enemy_classic", Name = "CIS", Archetype = "industrial_swarm", Units = new() { "b1-battle-droid" } },
                new() { Id = "separatist-extremists", Replaces = "enemy_guerrilla", Name = "Separatist Extremists", Archetype = "asymmetric", Units = new() { "commando-droid" } }
            },
            AssetReplacements = new TcAssetReplacements
            {
                Textures = new() { ["vanilla/faction_icon"] = "assets/republic_icon.png" },
                Audio = new() { ["audio/battle"] = "assets/clone_wars.ogg" }
            }
        };

        // ── TotalConversionValidator Tests ──────────────────────────────────

        [Fact]
        public void Validator_ValidManifest_IsValid()
        {
            var manifest = MakeStarWarsManifest();
            var result = TotalConversionValidator.Validate(manifest);
            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validator_WrongType_ReturnsError()
        {
            var manifest = MakeStarWarsManifest();
            manifest.Type = "content";
            var result = TotalConversionValidator.Validate(manifest);
            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("total_conversion"));
        }

        [Fact]
        public void Validator_NoReplacesVanilla_ReturnsError()
        {
            var manifest = MakeStarWarsManifest();
            manifest.ReplacesVanilla.Clear();
            var result = TotalConversionValidator.Validate(manifest);
            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("replaces_vanilla"));
        }

        [Fact]
        public void Validator_MissingFactionForReplacement_ReturnsError()
        {
            var manifest = MakeStarWarsManifest();
            manifest.ReplacesVanilla["player"] = "nonexistent-faction";
            var result = TotalConversionValidator.Validate(manifest);
            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("nonexistent-faction"));
        }

        [Fact]
        public void Validator_FactionMissingId_ReturnsError()
        {
            var manifest = MakeStarWarsManifest();
            manifest.Factions[0].Id = "";
            var result = TotalConversionValidator.Validate(manifest);
            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("missing id"));
        }

        [Fact]
        public void Validator_FactionMissingReplaces_ReturnsError()
        {
            var manifest = MakeStarWarsManifest();
            manifest.Factions[0].Replaces = "";
            var result = TotalConversionValidator.Validate(manifest);
            result.IsValid.Should().BeFalse();
        }

        [Fact]
        public void Validator_EmptyUnits_ReturnsWarning()
        {
            var manifest = MakeStarWarsManifest();
            manifest.Factions[0].Units.Clear();
            var result = TotalConversionValidator.Validate(manifest);
            result.Warnings.Should().Contain(w => w.Contains("no units"));
        }

        [Fact]
        public void Validator_UnknownArchetype_ReturnsWarning()
        {
            var manifest = MakeStarWarsManifest();
            manifest.Factions[0].Archetype = "robot-overlords";
            var result = TotalConversionValidator.Validate(manifest);
            result.Warnings.Should().Contain(w => w.Contains("robot-overlords"));
        }

        [Fact]
        public void Validator_UnknownVanillaFaction_ReturnsWarning()
        {
            var manifest = MakeStarWarsManifest();
            manifest.ReplacesVanilla["mystery-faction"] = "unknown-replacement";
            var result = TotalConversionValidator.Validate(manifest);
            result.Warnings.Should().Contain(w => w.Contains("Unknown vanilla faction"));
        }

        [Fact]
        public void Validator_SingletonFalse_ReturnsWarning()
        {
            var manifest = MakeStarWarsManifest();
            manifest.Singleton = false;
            var result = TotalConversionValidator.Validate(manifest);
            result.Warnings.Should().Contain(w => w.Contains("Singleton"));
        }

        [Fact]
        public void GetUnreplacedFactions_AllReplaced_ReturnsEmpty()
        {
            var manifest = MakeStarWarsManifest();
            var unreplaced = TotalConversionValidator.GetUnreplacedFactions(manifest);
            unreplaced.Should().BeEmpty();
        }

        [Fact]
        public void GetUnreplacedFactions_MissingOne_ReturnsMissing()
        {
            var manifest = MakeStarWarsManifest();
            manifest.ReplacesVanilla.Remove("enemy_guerrilla");
            manifest.Factions.RemoveAll(f => f.Replaces == "enemy_guerrilla");
            var unreplaced = TotalConversionValidator.GetUnreplacedFactions(manifest);
            unreplaced.Should().Contain("enemy_guerrilla");
        }

        [Fact]
        public void Validator_AllVanillaFactionsKnown()
        {
            TotalConversionValidator.VanillaFactionIds.Should().Contain("player");
            TotalConversionValidator.VanillaFactionIds.Should().Contain("enemy_classic");
            TotalConversionValidator.VanillaFactionIds.Should().Contain("enemy_guerrilla");
            TotalConversionValidator.VanillaFactionIds.Should().HaveCount(3);
        }

        // ── AssetReplacementEngine Tests ─────────────────────────────────────

        [Fact]
        public void AssetEngine_NoMapping_ReturnsVanillaPath()
        {
            var engine = new AssetReplacementEngine();
            string result = engine.ResolveTexture("textures/unit/soldier.png");
            result.Should().Be("textures/unit/soldier.png");
        }

        [Fact]
        public void AssetEngine_LoadFromManifest_RegistersMappings()
        {
            var engine = new AssetReplacementEngine();
            var manifest = MakeStarWarsManifest();
            engine.LoadFromManifest(manifest, "/tmp/fake-pack");
            engine.TotalMappings.Should().Be(2); // 1 texture + 1 audio
        }

        [Fact]
        public void AssetEngine_MappedAsset_MissingFile_FallsBackToVanilla()
        {
            var engine = new AssetReplacementEngine();
            var manifest = MakeStarWarsManifest();
            engine.LoadFromManifest(manifest, "/tmp/nonexistent-pack");
            // File doesn't exist → fallback to vanilla key
            string result = engine.ResolveTexture("vanilla/faction_icon");
            result.Should().Be("vanilla/faction_icon");
        }

        [Fact]
        public void AssetEngine_AudioMapping_RegisteredCorrectly()
        {
            var engine = new AssetReplacementEngine();
            var manifest = MakeStarWarsManifest();
            engine.LoadFromManifest(manifest, "");
            engine.GetAudioMap().Should().ContainKey("audio/battle");
            engine.GetAudioMap()["audio/battle"].Should().Be("assets/clone_wars.ogg");
        }

        [Fact]
        public void AssetEngine_TextureMapping_RegisteredCorrectly()
        {
            var engine = new AssetReplacementEngine();
            var manifest = MakeStarWarsManifest();
            engine.LoadFromManifest(manifest, "");
            engine.GetTextureMap().Should().ContainKey("vanilla/faction_icon");
            engine.GetTextureMap()["vanilla/faction_icon"].Should().Be("assets/republic_icon.png");
        }

        [Fact]
        public void AssetEngine_Clear_RemovesAllMappings()
        {
            var engine = new AssetReplacementEngine();
            var manifest = MakeStarWarsManifest();
            engine.LoadFromManifest(manifest, "");
            engine.TotalMappings.Should().BeGreaterThan(0);
            engine.Clear();
            engine.TotalMappings.Should().Be(0);
        }

        [Fact]
        public void AssetEngine_TextureAndUiAndAudio_AllResolve()
        {
            var engine = new AssetReplacementEngine();
            engine.ResolveTexture("any.png").Should().Be("any.png");
            engine.ResolveAudio("any.ogg").Should().Be("any.ogg");
            engine.ResolveUi("any.svg").Should().Be("any.svg");
        }

        [Fact]
        public void AssetEngine_UiMapping_RegisteredCorrectly()
        {
            var manifest = MakeStarWarsManifest();
            manifest.AssetReplacements.Ui["vanilla/ui_panel"] = "assets/sw_panel.png";
            var engine = new AssetReplacementEngine();
            engine.LoadFromManifest(manifest, "");
            engine.GetUiMap().Should().ContainKey("vanilla/ui_panel");
        }

        // ── TotalConversionManifest Model Tests ──────────────────────────────

        [Fact]
        public void Manifest_DefaultSingleton_IsTrue()
        {
            var manifest = new TotalConversionManifest();
            manifest.Singleton.Should().BeTrue();
        }

        [Fact]
        public void Manifest_DefaultType_IsTotalConversion()
        {
            var manifest = new TotalConversionManifest();
            manifest.Type.Should().Be("total_conversion");
        }

        [Fact]
        public void Manifest_FactionCount_ReflectsAdded()
        {
            var manifest = MakeStarWarsManifest();
            manifest.Factions.Should().HaveCount(3);
        }

        [Fact]
        public void Manifest_ReplacesVanilla_MapsCorrectly()
        {
            var manifest = MakeStarWarsManifest();
            manifest.ReplacesVanilla["player"].Should().Be("galactic-republic");
            manifest.ReplacesVanilla["enemy_classic"].Should().Be("confederacy-of-independent-systems");
            manifest.ReplacesVanilla["enemy_guerrilla"].Should().Be("separatist-extremists");
        }

        [Fact]
        public void Manifest_Empty_HasDefaults()
        {
            var manifest = new TotalConversionManifest();
            manifest.Version.Should().Be("0.1.0");
            manifest.FrameworkVersion.Should().Be(">=0.1.0 <1.0.0");
            manifest.Factions.Should().BeEmpty();
            manifest.ReplacesVanilla.Should().BeEmpty();
        }

        [Fact]
        public void TcFactionEntry_DefaultArchetype_IsCustom()
        {
            var faction = new TcFactionEntry();
            faction.Archetype.Should().Be("custom");
        }

        [Fact]
        public void TcAssetReplacements_Empty_HasEmptyDictionaries()
        {
            var replacements = new TcAssetReplacements();
            replacements.Textures.Should().BeEmpty();
            replacements.Audio.Should().BeEmpty();
            replacements.Ui.Should().BeEmpty();
        }
    }
}
