using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.SDK;
using DINOForge.SDK.Universe;
using FluentAssertions;
using Xunit;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for the Universe Bible system: CrosswalkDictionary, PackGenerator,
    /// UniverseLoader, NamingGuide, and related models.
    /// </summary>
    public class UniverseBibleTests
    {
        // YamlLoader provides centralized YAML deserialization

        // ──────────────────────── CrosswalkDictionary ────────────────────────

        [Fact]
        public void CrosswalkDictionary_ExactLookup_ReturnsEntry()
        {
            CrosswalkDictionary dict = CreateTestCrosswalk();

            CrosswalkEntry? entry = dict.LookupByVanillaId("militia_spearman");

            entry.Should().NotBeNull();
            entry!.ThemedId.Should().Be("clone_trooper_recruit");
            entry.ThemedName.Should().Be("Clone Trooper Recruit");
        }

        [Fact]
        public void CrosswalkDictionary_ExactLookup_CaseInsensitive()
        {
            CrosswalkDictionary dict = CreateTestCrosswalk();

            CrosswalkEntry? entry = dict.LookupByVanillaId("MILITIA_SPEARMAN");

            entry.Should().NotBeNull();
            entry!.ThemedId.Should().Be("clone_trooper_recruit");
        }

        [Fact]
        public void CrosswalkDictionary_ExactLookup_MissingEntry_ReturnsNull()
        {
            CrosswalkDictionary dict = CreateTestCrosswalk();

            CrosswalkEntry? entry = dict.LookupByVanillaId("nonexistent_unit");

            entry.Should().BeNull();
        }

        [Fact]
        public void CrosswalkDictionary_WildcardPattern_MatchesAndApplies()
        {
            CrosswalkDictionary dict = CreateTestCrosswalk();

            CrosswalkEntry? entry = dict.LookupByVanillaId("enemy_heavy_tank");

            entry.Should().NotBeNull();
            entry!.ThemedId.Should().Be("cis_heavy_tank");
            entry.VanillaId.Should().Be("enemy_heavy_tank");
        }

        [Fact]
        public void CrosswalkDictionary_WildcardPattern_NoMatch_ReturnsNull()
        {
            CrosswalkDictionary dict = CreateTestCrosswalk();

            CrosswalkEntry? entry = dict.LookupByVanillaId("friendly_unit");

            entry.Should().BeNull();
        }

        [Fact]
        public void CrosswalkDictionary_ReverseLookup_FindsVanillaId()
        {
            CrosswalkDictionary dict = CreateTestCrosswalk();

            string? vanillaId = dict.LookupByThemedId("clone_trooper_recruit");

            vanillaId.Should().Be("militia_spearman");
        }

        [Fact]
        public void CrosswalkDictionary_ReverseLookup_CaseInsensitive()
        {
            CrosswalkDictionary dict = CreateTestCrosswalk();

            string? vanillaId = dict.LookupByThemedId("CLONE_TROOPER_RECRUIT");

            vanillaId.Should().Be("militia_spearman");
        }

        [Fact]
        public void CrosswalkDictionary_ReverseLookup_WildcardPattern()
        {
            CrosswalkDictionary dict = CreateTestCrosswalk();

            string? vanillaId = dict.LookupByThemedId("cis_scout");

            vanillaId.Should().Be("enemy_scout");
        }

        [Fact]
        public void CrosswalkDictionary_GetAllEntries_ReturnsAll()
        {
            CrosswalkDictionary dict = CreateTestCrosswalk();

            IReadOnlyList<CrosswalkEntry> entries = dict.GetAllEntries();

            entries.Should().HaveCount(2);
        }

        [Fact]
        public void CrosswalkDictionary_StatModifiers_Preserved()
        {
            CrosswalkDictionary dict = CreateTestCrosswalk();

            CrosswalkEntry? entry = dict.LookupByVanillaId("militia_spearman");

            entry!.StatModifiers.Should().NotBeNull();
            entry.StatModifiers!["accuracy"].Should().Be(1.1f);
        }

        // ──────────────────────── NamingGuide ────────────────────────

        [Fact]
        public void NamingGuide_ApplyPrefix_Works()
        {
            NamingGuide guide = CreateTestNamingGuide();

            string result = guide.ApplyNaming("republic", "unit", "Soldier");

            result.Should().Be("Clone Soldier");
        }

        [Fact]
        public void NamingGuide_ApplyOverride_TakesPrecedence()
        {
            NamingGuide guide = CreateTestNamingGuide();

            string result = guide.ApplyNaming("republic", "weapon", "rifle");

            result.Should().Be("DC-15A Blaster Rifle");
        }

        [Fact]
        public void NamingGuide_NoMatchingFaction_FallsBackToGlobal()
        {
            NamingGuide guide = CreateTestNamingGuide();

            string result = guide.ApplyNaming("unknown_faction", "unit", "Soldier");

            // Global rules have no prefix/suffix, so base name returned
            result.Should().Be("Soldier");
        }

        [Fact]
        public void NamingGuide_PatternWithPlaceholder_Works()
        {
            NamingGuide guide = CreateTestNamingGuide();

            string result = guide.ApplyNaming("republic", "vehicle", "TE");

            result.Should().Be("AT-TE");
        }

        // ──────────────────────── UniverseLoader ────────────────────────

        [Fact]
        public void UniverseLoader_LoadFromYaml_ParsesCorrectly()
        {
            string yaml = @"
id: test-universe
name: Test Universe
description: A test universe
era: Test Era
version: '0.1.0'
";
            UniverseLoader loader = new UniverseLoader();

            UniverseBible bible = loader.LoadFromYaml(yaml);

            bible.Id.Should().Be("test-universe");
            bible.Name.Should().Be("Test Universe");
            bible.Era.Should().Be("Test Era");
            bible.Version.Should().Be("0.1.0");
        }

        [Fact]
        public void UniverseLoader_LoadFromDirectory_LoadsSeparateFiles()
        {
            string tempDir = Path.Combine(Path.GetTempPath(), $"universe_test_{Guid.NewGuid():N}");
            Directory.CreateDirectory(tempDir);
            try
            {
                // Write universe.yaml
                File.WriteAllText(Path.Combine(tempDir, "universe.yaml"),
                    "id: test-load\nname: Test Load\nversion: '0.1.0'\n");

                // Write crosswalk.yaml
                File.WriteAllText(Path.Combine(tempDir, "crosswalk.yaml"),
                    "entries:\n  unit_a:\n    vanilla_id: unit_a\n    themed_id: themed_a\n    themed_name: Themed A\n");

                // Write factions.yaml
                File.WriteAllText(Path.Combine(tempDir, "factions.yaml"),
                    "factions:\n  - id: faction_a\n    name: Faction A\n    alignment: Player\n    archetype: order\n");

                UniverseLoader loader = new UniverseLoader();
                UniverseBible bible = loader.LoadFromDirectory(tempDir);

                bible.Id.Should().Be("test-load");
                bible.CrosswalkDictionary.Entries.Should().HaveCount(1);
                bible.CrosswalkDictionary.Entries["unit_a"].ThemedId.Should().Be("themed_a");
                bible.FactionTaxonomy.Factions.Should().HaveCount(1);
                bible.FactionTaxonomy.Factions[0].Id.Should().Be("faction_a");
            }
            finally
            {
                Directory.Delete(tempDir, true);
            }
        }

        [Fact]
        public void UniverseLoader_MissingUniverseYaml_Throws()
        {
            string tempDir = Path.Combine(Path.GetTempPath(), $"universe_test_{Guid.NewGuid():N}");
            Directory.CreateDirectory(tempDir);
            try
            {
                UniverseLoader loader = new UniverseLoader();

                Action act = () => loader.LoadFromDirectory(tempDir);

                act.Should().Throw<FileNotFoundException>();
            }
            finally
            {
                Directory.Delete(tempDir, true);
            }
        }

        // ──────────────────────── PackGenerator ────────────────────────

        [Fact]
        public void PackGenerator_GeneratesManifestAndFiles()
        {
            string outputDir = Path.Combine(Path.GetTempPath(), $"packgen_test_{Guid.NewGuid():N}");
            try
            {
                UniverseBible bible = CreateTestBible();
                PackGenerator generator = new PackGenerator();

                PackGeneratorResult result = generator.Generate(bible, null, outputDir);

                result.GeneratedFiles.Should().HaveCountGreaterThanOrEqualTo(3);
                File.Exists(Path.Combine(outputDir, "pack.yaml")).Should().BeTrue();
                Directory.Exists(Path.Combine(outputDir, "factions")).Should().BeTrue();
                Directory.Exists(Path.Combine(outputDir, "units")).Should().BeTrue();
            }
            finally
            {
                if (Directory.Exists(outputDir))
                    Directory.Delete(outputDir, true);
            }
        }

        [Fact]
        public void PackGenerator_ManifestContainsCorrectId()
        {
            string outputDir = Path.Combine(Path.GetTempPath(), $"packgen_test_{Guid.NewGuid():N}");
            try
            {
                UniverseBible bible = CreateTestBible();
                PackGenerator generator = new PackGenerator();

                generator.Generate(bible, null, outputDir);

                string manifestContent = File.ReadAllText(Path.Combine(outputDir, "pack.yaml"));
                manifestContent.Should().Contain("test-bible");
            }
            finally
            {
                if (Directory.Exists(outputDir))
                    Directory.Delete(outputDir, true);
            }
        }

        [Fact]
        public void PackGenerator_FiltersByFaction()
        {
            string outputDir = Path.Combine(Path.GetTempPath(), $"packgen_test_{Guid.NewGuid():N}");
            try
            {
                UniverseBible bible = CreateTestBible();
                // Add a second faction
                bible.FactionTaxonomy.Factions.Add(new TaxonomyFaction
                {
                    Id = "faction_b",
                    Name = "Faction B",
                    Alignment = "Enemy",
                    Archetype = "industrial_swarm"
                });
                PackGenerator generator = new PackGenerator();

                PackGeneratorResult result = generator.Generate(
                    bible,
                    new List<string> { "test_faction" },
                    outputDir);

                // Should only have one faction file
                string[] factionFiles = Directory.GetFiles(Path.Combine(outputDir, "factions"), "*.yaml");
                factionFiles.Should().HaveCount(1);
                factionFiles[0].Should().Contain("test_faction");
            }
            finally
            {
                if (Directory.Exists(outputDir))
                    Directory.Delete(outputDir, true);
            }
        }

        // ──────────────────────── Pattern matching internals ────────────────────────

        [Theory]
        [InlineData("enemy_scout", "enemy_*", true)]
        [InlineData("enemy_heavy_tank", "enemy_*", true)]
        [InlineData("friendly_unit", "enemy_*", false)]
        [InlineData("building_barracks", "building_*", true)]
        [InlineData("unit_archer", "unit_*", true)]
        [InlineData("", "enemy_*", false)]
        public void MatchesPattern_VariousInputs_CorrectResult(string value, string pattern, bool expected)
        {
            bool result = CrosswalkDictionary.MatchesPattern(value, pattern);

            result.Should().Be(expected);
        }

        [Fact]
        public void ExtractWildcardValue_ExtractsCorrectPart()
        {
            string result = CrosswalkDictionary.ExtractWildcardValue("enemy_heavy_tank", "enemy_*");

            result.Should().Be("heavy_tank");
        }

        // ──────────────────────── Helper factories ────────────────────────

        private static CrosswalkDictionary CreateTestCrosswalk()
        {
            return new CrosswalkDictionary
            {
                Entries = new Dictionary<string, CrosswalkEntry>(StringComparer.OrdinalIgnoreCase)
                {
                    ["militia_spearman"] = new CrosswalkEntry
                    {
                        VanillaId = "militia_spearman",
                        ThemedId = "clone_trooper_recruit",
                        ThemedName = "Clone Trooper Recruit",
                        StatModifiers = new Dictionary<string, float> { ["accuracy"] = 1.1f }
                    },
                    ["line_swordsman"] = new CrosswalkEntry
                    {
                        VanillaId = "line_swordsman",
                        ThemedId = "clone_trooper",
                        ThemedName = "Clone Trooper"
                    }
                },
                Patterns = new List<CrosswalkPattern>
                {
                    new CrosswalkPattern
                    {
                        VanillaPattern = "enemy_*",
                        ThemedPattern = "cis_*",
                        ThemedNamePattern = "CIS *"
                    }
                }
            };
        }

        private static NamingGuide CreateTestNamingGuide()
        {
            return new NamingGuide
            {
                FactionRules = new Dictionary<string, FactionNamingRules>(StringComparer.OrdinalIgnoreCase)
                {
                    ["republic"] = new FactionNamingRules
                    {
                        Rules = new Dictionary<string, NamingRuleSet>(StringComparer.OrdinalIgnoreCase)
                        {
                            ["unit"] = new NamingRuleSet
                            {
                                Prefix = "Clone "
                            },
                            ["weapon"] = new NamingRuleSet
                            {
                                Overrides = new Dictionary<string, string>
                                {
                                    ["rifle"] = "DC-15A Blaster Rifle"
                                }
                            },
                            ["vehicle"] = new NamingRuleSet
                            {
                                Pattern = "AT-{name}"
                            }
                        }
                    }
                },
                GlobalRules = new NamingRuleSet()
            };
        }

        private static UniverseBible CreateTestBible()
        {
            return new UniverseBible
            {
                Id = "test-bible",
                Name = "Test Bible",
                Version = "0.1.0",
                FactionTaxonomy = new FactionTaxonomy
                {
                    Factions = new List<TaxonomyFaction>
                    {
                        new TaxonomyFaction
                        {
                            Id = "test_faction",
                            Name = "Test Faction",
                            Alignment = "Player",
                            Archetype = "order"
                        }
                    }
                },
                CrosswalkDictionary = new CrosswalkDictionary
                {
                    Entries = new Dictionary<string, CrosswalkEntry>(StringComparer.OrdinalIgnoreCase)
                    {
                        ["vanilla_unit"] = new CrosswalkEntry
                        {
                            VanillaId = "vanilla_unit",
                            ThemedId = "themed_unit",
                            ThemedName = "Themed Unit"
                        }
                    }
                },
                NamingGuide = new NamingGuide(),
                StyleGuide = new StyleGuide()
            };
        }

        // ──────────────────────── StyleGuide / FactionStyle ──────────────────

        [Fact]
        public void StyleGuide_DefaultsAreEmpty()
        {
            StyleGuide guide = new StyleGuide();
            guide.FactionStyles.Should().BeEmpty();
            guide.Global.Should().BeNull();
        }

        [Fact]
        public void FactionStyle_DefaultColorPaletteExists()
        {
            FactionStyle style = new FactionStyle();
            style.Colors.Should().NotBeNull();
            style.Colors.Primary.Should().Be("#FFFFFF");
            style.Colors.Secondary.Should().Be("#000000");
            style.Audio.Should().BeNull();
            style.Architecture.Should().BeNull();
            style.VisualTheme.Should().BeNull();
        }

        [Fact]
        public void ColorPalette_CanSetAllFields()
        {
            ColorPalette palette = new ColorPalette
            {
                Primary = "#FF0000",
                Secondary = "#00FF00",
                Tertiary = "#0000FF",
                UiHighlight = "#FFFF00"
            };
            palette.Primary.Should().Be("#FF0000");
            palette.Secondary.Should().Be("#00FF00");
            palette.Tertiary.Should().Be("#0000FF");
            palette.UiHighlight.Should().Be("#FFFF00");
        }

        [Fact]
        public void AudioTheme_CanSetAllFields()
        {
            AudioTheme theme = new AudioTheme
            {
                March = "march_theme",
                Ambient = "ambient_loop",
                Combat = "combat_intense",
                Victory = "victory_fanfare",
                Defeat = "defeat_dirge"
            };
            theme.March.Should().Be("march_theme");
            theme.Ambient.Should().Be("ambient_loop");
            theme.Combat.Should().Be("combat_intense");
            theme.Victory.Should().Be("victory_fanfare");
            theme.Defeat.Should().Be("defeat_dirge");
        }

        [Fact]
        public void StyleGuide_CanAddFactionStyles()
        {
            StyleGuide guide = new StyleGuide();
            guide.FactionStyles["republic"] = new FactionStyle
            {
                VisualTheme = "sleek_white",
                Architecture = "neoclassical",
                Colors = new ColorPalette { Primary = "#C0C0C0", Secondary = "#1E3A5F" },
                Audio = new AudioTheme { March = "republic_march", Combat = "republic_combat" }
            };
            guide.FactionStyles.Should().ContainKey("republic");
            guide.FactionStyles["republic"].VisualTheme.Should().Be("sleek_white");
            guide.FactionStyles["republic"].Audio!.March.Should().Be("republic_march");
        }

        [Fact]
        public void TaxonomySubFaction_CanSetFields()
        {
            TaxonomySubFaction sub = new TaxonomySubFaction
            {
                Id = "501st",
                Name = "501st Legion",
                Description = "Elite clone trooper unit",
                Specialization = "heavy_assault"
            };
            sub.Id.Should().Be("501st");
            sub.Name.Should().Be("501st Legion");
            sub.Description.Should().Be("Elite clone trooper unit");
            sub.Specialization.Should().Be("heavy_assault");
        }

        [Fact]
        public void FactionTaxonomy_CanAddFactionsWithSubFactions()
        {
            FactionTaxonomy taxonomy = new FactionTaxonomy();
            taxonomy.Factions.Add(new TaxonomyFaction
            {
                Id = "republic",
                Name = "Galactic Republic",
                Alignment = "Player",
                Archetype = "order",
                Description = "The democratic republic",
                SubFactions = new List<TaxonomySubFaction>
                {
                    new TaxonomySubFaction { Id = "501st", Name = "501st Legion" },
                    new TaxonomySubFaction { Id = "212th", Name = "212th Attack Battalion" }
                },
                UnitRoster = new Dictionary<string, string> { ["line_infantry"] = "clone_trooper" }
            });
            taxonomy.Factions.Should().HaveCount(1);
            taxonomy.Factions[0].SubFactions.Should().HaveCount(2);
            taxonomy.Factions[0].UnitRoster.Should().ContainKey("line_infantry");
        }

        // ──────────────────────── PackGeneratorResult ────────────────────────

        [Fact]
        public void PackGeneratorResult_IsClean_WhenNoWarnings()
        {
            string outputDir = Path.Combine(Path.GetTempPath(), $"packgen_clean_{Guid.NewGuid():N}");
            try
            {
                UniverseBible bible = CreateTestBible();
                PackGenerator generator = new PackGenerator();
                PackGeneratorResult result = generator.Generate(bible, null, outputDir);

                result.IsClean.Should().BeTrue("generated pack had no warnings");
                result.Warnings.Should().BeEmpty();
            }
            finally
            {
                if (Directory.Exists(outputDir))
                    Directory.Delete(outputDir, true);
            }
        }

        // ──────────────────────── GlobalStyle ────────────────────────────────

        [Fact]
        public void GlobalStyle_CanSetAllFields()
        {
            DINOForge.SDK.Universe.GlobalStyle gs = new DINOForge.SDK.Universe.GlobalStyle
            {
                Tone = "gritty",
                UiTheme = "dark_metal",
                FontStyle = "condensed_sans"
            };
            gs.Tone.Should().Be("gritty");
            gs.UiTheme.Should().Be("dark_metal");
            gs.FontStyle.Should().Be("condensed_sans");
        }

        [Fact]
        public void GlobalStyle_DefaultsAreNull()
        {
            DINOForge.SDK.Universe.GlobalStyle gs = new DINOForge.SDK.Universe.GlobalStyle();
            gs.Tone.Should().BeNull();
            gs.UiTheme.Should().BeNull();
            gs.FontStyle.Should().BeNull();
        }

        [Fact]
        public void StyleGuide_WithGlobal_RoundtripsThroughModel()
        {
            StyleGuide guide = new StyleGuide
            {
                Global = new DINOForge.SDK.Universe.GlobalStyle { Tone = "epic", UiTheme = "gold_empire" }
            };
            guide.Global.Should().NotBeNull();
            guide.Global!.Tone.Should().Be("epic");
        }
    }
}
