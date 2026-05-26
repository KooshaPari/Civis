// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Task #210 Phase 3 — UIContentLoader JsonGuard wiring negative tests.
// Mirrors PackLoaderTests.cs Pattern #75 / Pattern #86 negative-test pattern.

using System;
using System.IO;
using DINOForge.Domains.UI;
using DINOForge.Domains.UI.Registries;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Pins the JsonGuard.ValidateOrThrow wiring at the three UIContentLoader
    /// deserialize sites (HUD elements, menus, themes). UIContentLoader wraps
    /// every load failure in <see cref="InvalidOperationException"/>; the
    /// underlying validation surface is the
    /// <see cref="System.IO.InvalidDataException"/> carried as InnerException.
    ///
    /// These negative tests enforce that:
    ///   - HudElementDefinition.Validate() rejects blank id
    ///   - MenuDefinition.Validate() rejects blank title
    ///   - ThemeDefinition.Validate() rejects malformed hex colors
    /// at the deserialize site, not later when Register() runs.
    /// </summary>
    public class UIContentLoaderValidationTests : IDisposable
    {
        private readonly string _packDir;
        private readonly UIContentLoader _loader;
        private readonly HudElementRegistry _hudElementRegistry;
        private readonly MenuRegistry _menuRegistry;
        private readonly ThemeRegistry _themeRegistry;

        public UIContentLoaderValidationTests()
        {
            _packDir = Path.Combine(
                Path.GetTempPath(),
                "dinoforge-uicontentloader-tests-" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_packDir);

            _hudElementRegistry = new HudElementRegistry();
            _menuRegistry = new MenuRegistry();
            _themeRegistry = new ThemeRegistry();
            _loader = new UIContentLoader(_hudElementRegistry, _menuRegistry, _themeRegistry);
        }

        public void Dispose()
        {
            try
            {
                if (Directory.Exists(_packDir))
                {
                    Directory.Delete(_packDir, recursive: true);
                }
            }
            catch (IOException)
            {
                // Best-effort cleanup; leave the temp dir if locked by an antivirus etc.
            }
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void UIContentLoader_RejectsHudElementWithBlankId()
        {
            // Arrange — author a hud_elements/*.yaml with one valid + one blank-id element.
            string elementsDir = Path.Combine(_packDir, "hud_elements");
            Directory.CreateDirectory(elementsDir);
            string yaml = @"
hud_elements:
  - id: ''
    type: health_bar
    position: top_left
    width: 200
    height: 50
";
            File.WriteAllText(Path.Combine(elementsDir, "bad-hud.yaml"), yaml);

            // Act
            Action act = () => _loader.LoadPack(_packDir, "bad-hud-pack");

            // Assert — UIContentLoader wraps in InvalidOperationException; the
            // semantic violation surfaces as the InnerException with a path-prefixed
            // message that names the offending field.
            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<InvalidDataException>()
                .WithMessage("*id*");

            // Side-effect: nothing should have been registered.
            _hudElementRegistry.Count.Should().Be(0);
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void UIContentLoader_RejectsMenuWithMissingTitle()
        {
            // Arrange — menu with empty title; YamlDotNet treats '' as a present-but-blank
            // string, which is the case the IValidatable Validate() must catch (schema
            // alone cannot distinguish blank-string from "missing required field").
            string menusDir = Path.Combine(_packDir, "menus");
            Directory.CreateDirectory(menusDir);
            string yaml = @"
menus:
  - id: blank-title-menu
    title: ''
    items:
      - id: ok-item
        label: 'Click me'
        action: navigate
        target: somewhere
";
            File.WriteAllText(Path.Combine(menusDir, "bad-menu.yaml"), yaml);

            // Act
            Action act = () => _loader.LoadPack(_packDir, "bad-menu-pack");

            // Assert — title violation surfaces from MenuDefinition.Validate()
            // through JsonGuard, wrapped by UIContentLoader's catch.
            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<InvalidDataException>()
                .WithMessage("*title*");

            _menuRegistry.Count.Should().Be(0);
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void UIContentLoader_RejectsThemeWithInvalidColor()
        {
            // Arrange — theme with a malformed hex color (missing '#', wrong digit count).
            string themesDir = Path.Combine(_packDir, "themes");
            Directory.CreateDirectory(themesDir);
            string yaml = @"
themes:
  - id: bad-theme
    name: Bad Theme
    primary_color: 'NOT-A-HEX'
    secondary_color: '#666666'
    accent_color: '#FF6B00'
";
            File.WriteAllText(Path.Combine(themesDir, "bad-theme.yaml"), yaml);

            // Snapshot the pre-LoadPack state — ThemeRegistry seeds 2 default
            // themes ("dark-theme", "light-theme") in its ctor, so we assert no
            // *additional* themes leaked through validation rather than asserting
            // Count == 0.
            int baseline = _themeRegistry.Count;

            // Act
            Action act = () => _loader.LoadPack(_packDir, "bad-theme-pack");

            // Assert — primary_color violation surfaces from ThemeDefinition.Validate()
            // through JsonGuard, wrapped by UIContentLoader's catch.
            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<InvalidDataException>()
                .WithMessage("*primary_color*");

            // The bad-theme YAML must not have added the malformed entry.
            _themeRegistry.Count.Should().Be(baseline);
            _themeRegistry.Contains("bad-theme").Should().BeFalse();
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void UIContentLoader_RejectsHudElementWithOutOfRangeOpacity()
        {
            // Pin the range-bound branch in HudElementDefinition.Validate(): opacity
            // must be in [0.0, 1.0]. CanvasGroup.alpha clamps anyway, but the loader
            // refuses to register the violation rather than silently truncating.
            string elementsDir = Path.Combine(_packDir, "hud_elements");
            Directory.CreateDirectory(elementsDir);
            string yaml = @"
hud_elements:
  - id: bright-hud
    type: alert_banner
    position: center
    width: 400
    height: 60
    opacity: 1.5
";
            File.WriteAllText(Path.Combine(elementsDir, "bad-opacity.yaml"), yaml);

            Action act = () => _loader.LoadPack(_packDir, "bad-opacity-pack");

            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<InvalidDataException>()
                .WithMessage("*opacity*");

            _hudElementRegistry.Count.Should().Be(0);
        }
    }
}
