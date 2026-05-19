using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;
using static DINOForge.SDK.Models.TotalConversionManifest;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="AssetReplacementEngine"/> covering mapping lifecycle,
    /// resolution behavior, and fallback logic.
    /// </summary>
    public class AssetReplacementEngineUnitTests : IDisposable
    {
        private readonly string _tempDir;
        private readonly AssetReplacementEngine _engine;

        public AssetReplacementEngineUnitTests()
        {
            _tempDir = Path.Combine(Path.GetTempPath(), "asset_replacement_tests_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempDir);
            _engine = new AssetReplacementEngine();
        }

        public void Dispose()
        {
            try
            {
                if (Directory.Exists(_tempDir))
                    Directory.Delete(_tempDir, recursive: true);
            }
            catch { /* best-effort cleanup */ }
        }

        [Fact]
        public void LoadFromManifest_WithEmptyReplacements_TotalMappingsIsZero()
        {
            // Arrange
            var manifest = new TotalConversionManifest
            {
                Id = "test-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string>(),
                    Audio = new Dictionary<string, string>(),
                    Ui = new Dictionary<string, string>()
                }
            };

            // Act
            _engine.LoadFromManifest(manifest, _tempDir);

            // Assert
            _engine.TotalMappings.Should().Be(0);
            _engine.GetTextureMap().Should().BeEmpty();
            _engine.GetAudioMap().Should().BeEmpty();
            _engine.GetUiMap().Should().BeEmpty();
        }

        [Fact]
        public void LoadFromManifest_WithSingleMapping_PopulatesCorrectMap()
        {
            // Arrange
            var manifest = new TotalConversionManifest
            {
                Id = "test-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string> { { "vanilla/texture.png", "mod/new_texture.png" } },
                    Audio = new Dictionary<string, string>(),
                    Ui = new Dictionary<string, string>()
                }
            };

            // Act
            _engine.LoadFromManifest(manifest, _tempDir);

            // Assert
            _engine.TotalMappings.Should().Be(1);
            _engine.GetTextureMap().Should().ContainKey("vanilla/texture.png");
        }

        [Fact]
        public void ResolveTexture_WithNoMapping_ReturnsSameVanillaPath()
        {
            // Arrange
            var manifest = new TotalConversionManifest
            {
                Id = "test-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string>(),
                    Audio = new Dictionary<string, string>(),
                    Ui = new Dictionary<string, string>()
                }
            };
            _engine.LoadFromManifest(manifest, _tempDir);
            string vanillaPath = "vanilla/missing_texture.png";

            // Act
            string result = _engine.ResolveTexture(vanillaPath);

            // Assert
            result.Should().Be(vanillaPath);
        }

        [Fact]
        public void ResolveTexture_WithMappingButMissingFile_ReturnsFallbackVanillaPath()
        {
            // Arrange
            var manifest = new TotalConversionManifest
            {
                Id = "test-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string> { { "vanilla/texture.png", "nonexistent/mod_texture.png" } },
                    Audio = new Dictionary<string, string>(),
                    Ui = new Dictionary<string, string>()
                }
            };
            _engine.LoadFromManifest(manifest, _tempDir);
            string vanillaPath = "vanilla/texture.png";

            // Act
            string result = _engine.ResolveTexture(vanillaPath);

            // Assert
            result.Should().Be(vanillaPath);
        }

        [Fact]
        public void ResolveTexture_WithMappingAndExistingFile_ReturnsModPath()
        {
            // Arrange
            string modFilePath = Path.Combine(_tempDir, "mod_texture.png");
            File.WriteAllText(modFilePath, "texture content");

            var manifest = new TotalConversionManifest
            {
                Id = "test-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string> { { "vanilla/texture.png", "mod_texture.png" } },
                    Audio = new Dictionary<string, string>(),
                    Ui = new Dictionary<string, string>()
                }
            };
            _engine.LoadFromManifest(manifest, _tempDir);

            // Act
            string result = _engine.ResolveTexture("vanilla/texture.png");

            // Assert
            result.Should().Be(modFilePath);
        }

        [Fact]
        public void ResolveAudio_WithValidMapping_ReturnsModPath()
        {
            // Arrange
            string audioFilePath = Path.Combine(_tempDir, "mod_sound.wav");
            File.WriteAllText(audioFilePath, "audio data");

            var manifest = new TotalConversionManifest
            {
                Id = "test-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string>(),
                    Audio = new Dictionary<string, string> { { "vanilla/sound.wav", "mod_sound.wav" } },
                    Ui = new Dictionary<string, string>()
                }
            };
            _engine.LoadFromManifest(manifest, _tempDir);

            // Act
            string result = _engine.ResolveAudio("vanilla/sound.wav");

            // Assert
            result.Should().Be(audioFilePath);
        }

        [Fact]
        public void ResolveUi_WithValidMapping_ReturnsModPath()
        {
            // Arrange
            string uiFilePath = Path.Combine(_tempDir, "mod_ui.prefab");
            File.WriteAllText(uiFilePath, "ui data");

            var manifest = new TotalConversionManifest
            {
                Id = "test-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string>(),
                    Audio = new Dictionary<string, string>(),
                    Ui = new Dictionary<string, string> { { "vanilla/ui.prefab", "mod_ui.prefab" } }
                }
            };
            _engine.LoadFromManifest(manifest, _tempDir);

            // Act
            string result = _engine.ResolveUi("vanilla/ui.prefab");

            // Assert
            result.Should().Be(uiFilePath);
        }

        [Fact]
        public void ResolveTexture_WithAbsolutePath_UsesPathDirectlyIfExists()
        {
            // Arrange
            string absolutePath = Path.Combine(_tempDir, "absolute_texture.png");
            File.WriteAllText(absolutePath, "texture");

            var manifest = new TotalConversionManifest
            {
                Id = "test-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string> { { "vanilla/texture.png", absolutePath } },
                    Audio = new Dictionary<string, string>(),
                    Ui = new Dictionary<string, string>()
                }
            };
            _engine.LoadFromManifest(manifest, _tempDir);

            // Act
            string result = _engine.ResolveTexture("vanilla/texture.png");

            // Assert
            result.Should().Be(absolutePath);
        }

        [Fact]
        public void ResolveTexture_WithAbsolutePath_FallsBackIfNotExists()
        {
            // Arrange
            string absolutePath = Path.Combine(_tempDir, "nonexistent_absolute.png");

            var manifest = new TotalConversionManifest
            {
                Id = "test-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string> { { "vanilla/texture.png", absolutePath } },
                    Audio = new Dictionary<string, string>(),
                    Ui = new Dictionary<string, string>()
                }
            };
            _engine.LoadFromManifest(manifest, _tempDir);

            // Act
            string result = _engine.ResolveTexture("vanilla/texture.png");

            // Assert
            result.Should().Be("vanilla/texture.png");
        }

        [Fact]
        public void Clear_RemovesAllMappings()
        {
            // Arrange
            var manifest = new TotalConversionManifest
            {
                Id = "test-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string> { { "key1", "val1" } },
                    Audio = new Dictionary<string, string> { { "key2", "val2" } },
                    Ui = new Dictionary<string, string> { { "key3", "val3" } }
                }
            };
            _engine.LoadFromManifest(manifest, _tempDir);
            _engine.TotalMappings.Should().Be(3);

            // Act
            _engine.Clear();

            // Assert
            _engine.TotalMappings.Should().Be(0);
            _engine.GetTextureMap().Should().BeEmpty();
            _engine.GetAudioMap().Should().BeEmpty();
            _engine.GetUiMap().Should().BeEmpty();
        }

        [Fact]
        public void ResolveTexture_IsCaseInsensitive()
        {
            // Arrange
            string modFilePath = Path.Combine(_tempDir, "MOD_TEXTURE.PNG");
            File.WriteAllText(modFilePath, "texture");

            var manifest = new TotalConversionManifest
            {
                Id = "test-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string> { { "VANILLA/TEXTURE.PNG", "MOD_TEXTURE.PNG" } },
                    Audio = new Dictionary<string, string>(),
                    Ui = new Dictionary<string, string>()
                }
            };
            _engine.LoadFromManifest(manifest, _tempDir);

            // Act — lookup with different casing
            string result = _engine.ResolveTexture("vanilla/texture.png");

            // Assert
            result.Should().Be(modFilePath);
        }

        [Fact]
        public void LoadFromManifest_WithMultipleMappings_CountsAllTypes()
        {
            // Arrange
            var manifest = new TotalConversionManifest
            {
                Id = "test-pack",
                AssetReplacements = new TcAssetReplacements
                {
                    Textures = new Dictionary<string, string>
                    {
                        { "vanilla/tex1.png", "mod/tex1.png" },
                        { "vanilla/tex2.png", "mod/tex2.png" }
                    },
                    Audio = new Dictionary<string, string>
                    {
                        { "vanilla/sound.wav", "mod/sound.wav" }
                    },
                    Ui = new Dictionary<string, string>
                    {
                        { "vanilla/ui1.prefab", "mod/ui1.prefab" },
                        { "vanilla/ui2.prefab", "mod/ui2.prefab" },
                        { "vanilla/ui3.prefab", "mod/ui3.prefab" }
                    }
                }
            };

            // Act
            _engine.LoadFromManifest(manifest, _tempDir);

            // Assert
            _engine.TotalMappings.Should().Be(6);
            _engine.GetTextureMap().Should().HaveCount(2);
            _engine.GetAudioMap().Should().HaveCount(1);
            _engine.GetUiMap().Should().HaveCount(3);
        }
    }
}
