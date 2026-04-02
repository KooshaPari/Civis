using System;
using DINOForge.SDK;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for pack loading and manifest parsing.
    /// Maps to: US-F1.1 Pack Manifest Loading
    /// </summary>
    [Trait("Category", "UserStory")]
    [Trait("UserStory", "US-F1.1")]
    [Trait("Category", "Epic")]
    [Trait("Epic", "Epic-PackSystem")]
    public class PackLoaderTests
    {
        private readonly PackLoader _loader = new();

        [Fact]
        [Trait("Journey", "Journey-InstallPlay")]
        public void LoadFromString_ValidManifest_DeserializesCorrectly()
        {
            string yaml = @"
id: test-pack
name: Test Pack
version: 0.1.0
framework_version: '>=0.1.0'
author: Test Author
type: content
depends_on:
  - core
conflicts_with: []
loads:
  factions:
    - republic
    - cis
";
            PackManifest manifest = _loader.LoadFromString(yaml);

            manifest.Id.Should().Be("test-pack");
            manifest.Name.Should().Be("Test Pack");
            manifest.Version.Should().Be("0.1.0");
            manifest.Author.Should().Be("Test Author");
            manifest.Type.Should().Be("content");
            manifest.DependsOn.Should().Contain("core");
            manifest.Loads.Should().NotBeNull();
            manifest.Loads!.Factions.Should().Contain("republic");
            manifest.Loads!.Factions.Should().Contain("cis");
        }

        [Fact]
        public void LoadFromString_MissingId_Throws()
        {
            string yaml = @"
name: No ID Pack
version: 0.1.0
author: Test
type: content
";
            Action act = () => _loader.LoadFromString(yaml);
            act.Should().Throw<InvalidOperationException>()
                .WithMessage("*id*");
        }

        [Fact]
        public void LoadFromString_MissingName_Throws()
        {
            string yaml = @"
id: no-name
version: 0.1.0
author: Test
type: content
";
            Action act = () => _loader.LoadFromString(yaml);
            act.Should().Throw<InvalidOperationException>()
                .WithMessage("*name*");
        }

        [Fact]
        public void LoadFromString_MinimalManifest_UsesDefaults()
        {
            string yaml = @"
id: minimal
name: Minimal Pack
version: 0.1.0
author: Test
type: balance
";
            PackManifest manifest = _loader.LoadFromString(yaml);

            manifest.LoadOrder.Should().Be(100);
            manifest.DependsOn.Should().BeEmpty();
            manifest.ConflictsWith.Should().BeEmpty();
            manifest.Loads.Should().BeNull();
        }

        [Fact]
        public void LoadFromString_TotalConversion_ParsesFullManifest()
        {
            string yaml = @"
id: starwars-republic-cis
name: Republic vs CIS Pack
version: 0.1.0
framework_version: '>=0.4.0 <0.5.0'
author: DINOForge Agents
type: total_conversion
depends_on:
  - dino-core
  - dino-warfare-domain
conflicts_with:
  - modern-west-pack
load_order: 50
loads:
  factions:
    - republic
    - cis
  doctrines:
    - elite_discipline
    - mechanized_attrition
  audio:
    - sw_blaster_pack
  visuals:
    - sw_projectiles
  localization:
    - sw_english
";
            PackManifest manifest = _loader.LoadFromString(yaml);

            manifest.Type.Should().Be("total_conversion");
            manifest.LoadOrder.Should().Be(50);
            manifest.DependsOn.Should().HaveCount(2);
            manifest.ConflictsWith.Should().Contain("modern-west-pack");
            manifest.Loads!.Doctrines.Should().Contain("mechanized_attrition");
            manifest.Loads!.Audio.Should().Contain("sw_blaster_pack");
        }

        [Fact]
        public void LoadFromFile_NonexistentFile_Throws()
        {
            Action act = () => _loader.LoadFromFile("/nonexistent/path/pack.yaml");
            act.Should().Throw<System.IO.FileNotFoundException>();
        }

        // ── Edge Cases ───────────────────────────────────────────────────

        [Fact]
        public void LoadFromString_EmptyString_Throws()
        {
            Action act = () => _loader.LoadFromString("");
            act.Should().Throw<Exception>();
        }

        [Fact]
        public void LoadFromString_WhitespaceOnly_Throws()
        {
            Action act = () => _loader.LoadFromString("   \n  \n  ");
            act.Should().Throw<Exception>();
        }

        [Fact]
        public void LoadFromString_UnknownFields_IgnoredGracefully()
        {
            string yaml = @"
id: unknown-fields-pack
name: Unknown Fields Pack
version: 0.1.0
author: Test
type: content
unknown_field_1: some value
another_unknown:
  nested: true
  values:
    - a
    - b
";
            PackManifest manifest = _loader.LoadFromString(yaml);

            manifest.Id.Should().Be("unknown-fields-pack");
            manifest.Name.Should().Be("Unknown Fields Pack");
        }

        [Fact]
        public void LoadFromString_UnicodePackName_ParsesCorrectly()
        {
            string yaml = @"
id: unicode-pack
name: '\u6E2C\u8A66\u30D1\u30C3\u30AF \u2605\u2606\u2605'
version: 0.1.0
author: Test
type: content
";
            PackManifest manifest = _loader.LoadFromString(yaml);

            manifest.Id.Should().Be("unicode-pack");
            manifest.Name.Should().NotBeNullOrEmpty();
        }

        [Fact]
        public void LoadFromString_VeryLongFieldValues_ParsesCorrectly()
        {
            string longName = new string('A', 500);
            string longDesc = new string('B', 2000);
            string yaml = $@"
id: long-values
name: {longName}
version: 0.1.0
author: Test
type: content
description: {longDesc}
";
            PackManifest manifest = _loader.LoadFromString(yaml);

            manifest.Name.Should().HaveLength(500);
            manifest.Description.Should().HaveLength(2000);
        }

        [Fact]
        public void LoadFromString_NestedLoads_MultipleContentTypes()
        {
            string yaml = @"
id: multi-loads
name: Multi Loads Pack
version: 0.1.0
author: Test
type: total_conversion
loads:
  factions:
    - factions/republic
    - factions/cis
  units:
    - units/infantry
    - units/vehicles
  buildings:
    - buildings/military
    - buildings/economy
  weapons:
    - weapons/blasters
  doctrines:
    - doctrines/elite
  wave_templates:
    - waves/campaign
  scenarios:
    - scenarios/main
";
            PackManifest manifest = _loader.LoadFromString(yaml);

            manifest.Loads.Should().NotBeNull();
            manifest.Loads!.Factions.Should().HaveCount(2);
            manifest.Loads!.Units.Should().HaveCount(2);
            manifest.Loads!.Buildings.Should().HaveCount(2);
            manifest.Loads!.Weapons.Should().HaveCount(1);
            manifest.Loads!.Doctrines.Should().HaveCount(1);
            manifest.Loads!.WaveTemplates.Should().HaveCount(1);
            manifest.Loads!.Scenarios.Should().HaveCount(1);
        }

        [Fact]
        public void LoadFromString_VersionFormats_Accepted()
        {
            string yaml = @"
id: version-test
name: Version Test
version: 1.2.3
author: Test
type: content
";
            PackManifest manifest = _loader.LoadFromString(yaml);
            manifest.Version.Should().Be("1.2.3");
        }

        [Fact]
        public void LoadFromString_MissingVersion_Throws()
        {
            string yaml = @"
id: no-version
name: No Version
author: Test
type: content
version: ''
";
            Action act = () => _loader.LoadFromString(yaml);
            act.Should().Throw<InvalidOperationException>()
                .WithMessage("*version*");
        }

        [Fact]
        public void LoadFromString_OverridesSection_ParsesCorrectly()
        {
            string yaml = @"
id: override-pack
name: Override Pack
version: 0.1.0
author: Test
type: balance
overrides:
  units:
    - overrides/units
  stats:
    - overrides/balance
";
            PackManifest manifest = _loader.LoadFromString(yaml);

            manifest.Overrides.Should().NotBeNull();
            manifest.Overrides!.Units.Should().HaveCount(1);
            manifest.Overrides!.Stats.Should().HaveCount(1);
        }
    }
}
