// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Task #210 Phase 6 / task #241 — UniverseLoader JsonGuard wiring negative tests.
// Mirrors PackLoaderTests.cs / UIContentLoaderValidationTests.cs / EconomyContentLoaderValidationTests.cs
// Pattern #75 / Pattern #86 negative-test pattern.

using System;
using System.IO;
using DINOForge.SDK.Universe;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Pins the <c>JsonGuard.ValidateOrThrow</c> wiring at the six
    /// <see cref="UniverseLoader"/> deserialize sites:
    ///   1. <see cref="UniverseLoader.LoadFromDirectory(string)"/> — universe.yaml
    ///   2. <see cref="UniverseLoader.LoadFromYaml(string)"/>
    ///   3. crosswalk.yaml side-load
    ///   4. factions.yaml side-load
    ///   5. naming.yaml side-load
    ///   6. style.yaml side-load
    ///
    /// Each negative test asserts that <see cref="InvalidDataException"/> is thrown
    /// at the deserialize site (Phase 6 of the #210 sweep) rather than failing
    /// silently or surfacing later.
    /// </summary>
    public class UniverseLoaderValidationTests : IDisposable
    {
        private readonly string _tempDir;
        private readonly UniverseLoader _loader;

        public UniverseLoaderValidationTests()
        {
            _tempDir = Path.Combine(
                Path.GetTempPath(),
                "dinoforge-universeloader-tests-" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempDir);
            _loader = new UniverseLoader();
        }

        public void Dispose()
        {
            if (Directory.Exists(_tempDir))
            {
                Directory.Delete(_tempDir, recursive: true);
            }
        }

        // ── Site 1+2: UniverseBible (LoadFromYaml + LoadFromDirectory) ───

        [Fact]
        [Trait("Category", "Validation")]
        public void UniverseLoader_RejectsUniverseBibleWithBlankId()
        {
            string yaml = @"
id: ''
name: Blank Id Universe
version: '0.1.0'
";
            Action act = () => _loader.LoadFromYaml(yaml);
            act.Should().Throw<InvalidDataException>()
                .WithMessage("*id*");
        }

        // ── Site 3: CrosswalkDictionary (crosswalk.yaml) ─────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void UniverseLoader_RejectsCrosswalkDictionaryWithBlankEntryThemedId()
        {
            // Valid universe.yaml so we reach the crosswalk side-load.
            File.WriteAllText(
                Path.Combine(_tempDir, "universe.yaml"),
                "id: ok-universe\nname: OK Universe\nversion: '0.1.0'\n");

            // Crosswalk entry with blank themed_id — should fail validation.
            File.WriteAllText(
                Path.Combine(_tempDir, "crosswalk.yaml"),
                "entries:\n  vanilla_a:\n    vanilla_id: vanilla_a\n    themed_id: ''\n");

            Action act = () => _loader.LoadFromDirectory(_tempDir);
            act.Should().Throw<InvalidDataException>()
                .WithMessage("*themed_id*");
        }

        // ── Site 4: FactionTaxonomy (factions.yaml) ──────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void UniverseLoader_RejectsFactionTaxonomyWithBlankFactionId()
        {
            File.WriteAllText(
                Path.Combine(_tempDir, "universe.yaml"),
                "id: ok-universe\nname: OK Universe\nversion: '0.1.0'\n");

            // Faction with blank id — should fail validation.
            File.WriteAllText(
                Path.Combine(_tempDir, "factions.yaml"),
                "factions:\n  - id: ''\n    name: Nameless Faction\n    alignment: Player\n    archetype: order\n");

            Action act = () => _loader.LoadFromDirectory(_tempDir);
            act.Should().Throw<InvalidDataException>()
                .WithMessage("*id*");
        }

        // ── Site 5: NamingGuide (naming.yaml) ────────────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void UniverseLoader_RejectsNamingGuideWithBlankFactionRuleKey()
        {
            File.WriteAllText(
                Path.Combine(_tempDir, "universe.yaml"),
                "id: ok-universe\nname: OK Universe\nversion: '0.1.0'\n");

            // Faction-rule key is blank — should fail validation.
            File.WriteAllText(
                Path.Combine(_tempDir, "naming.yaml"),
                "faction_rules:\n  '':\n    rules:\n      unit:\n        prefix: 'X-'\n");

            Action act = () => _loader.LoadFromDirectory(_tempDir);
            act.Should().Throw<InvalidDataException>()
                .WithMessage("*faction_rules*");
        }

        // ── Site 6: StyleGuide (style.yaml) ──────────────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void UniverseLoader_RejectsStyleGuideWithBlankColorPrimary()
        {
            File.WriteAllText(
                Path.Combine(_tempDir, "universe.yaml"),
                "id: ok-universe\nname: OK Universe\nversion: '0.1.0'\n");

            // Faction style has a blank colors.primary — should fail validation.
            File.WriteAllText(
                Path.Combine(_tempDir, "style.yaml"),
                "faction_styles:\n  republic:\n    colors:\n      primary: ''\n      secondary: '#1A3A6B'\n");

            Action act = () => _loader.LoadFromDirectory(_tempDir);
            act.Should().Throw<InvalidDataException>()
                .WithMessage("*primary*");
        }

        // ── Bonus: full UniverseBible LoadFromDirectory blank-id rejection ───

        [Fact]
        [Trait("Category", "Validation")]
        public void UniverseLoader_LoadFromDirectory_RejectsUniverseBibleWithBlankName()
        {
            File.WriteAllText(
                Path.Combine(_tempDir, "universe.yaml"),
                "id: ok-id\nname: ''\nversion: '0.1.0'\n");

            Action act = () => _loader.LoadFromDirectory(_tempDir);
            act.Should().Throw<InvalidDataException>()
                .WithMessage("*name*");
        }
    }
}
