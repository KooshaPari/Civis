using System.Collections.Generic;
using DINOForge.SDK.Universe;
using FluentAssertions;
using Xunit;

namespace DINOForge.SDK.Tests.SDK;

public sealed class CrosswalkDictionaryCoverageTests
{
    [Fact]
    public void LookupByVanillaId_and_LookupByThemedId_should_support_exact_and_wildcard_mappings()
    {
        CrosswalkDictionary dictionary = new CrosswalkDictionary();

        CrosswalkEntry exactEntry = new CrosswalkEntry
        {
            VanillaId = "enemy_scout",
            ThemedId = "cis_scout",
            ThemedName = "CIS Scout"
        };

        dictionary.Entries["enemy_scout"] = exactEntry;
        dictionary.Patterns.Add(new CrosswalkPattern
        {
            VanillaPattern = "enemy_*",
            ThemedPattern = "cis_*",
            ThemedNamePattern = "CIS *",
            ThemedDescriptionPattern = "Converted *",
            StatModifiers = new Dictionary<string, float>
            {
                ["health"] = 1.25f
            }
        });

        CrosswalkEntry? exactLookup = dictionary.LookupByVanillaId("ENEMY_SCOUT");
        exactLookup.Should().BeSameAs(exactEntry);

        CrosswalkEntry? wildcardLookup = dictionary.LookupByVanillaId("enemy_archer");
        wildcardLookup.Should().NotBeNull();
        wildcardLookup!.VanillaId.Should().Be("enemy_archer");
        wildcardLookup.ThemedId.Should().Be("cis_archer");
        wildcardLookup.ThemedName.Should().Be("CIS archer");
        wildcardLookup.ThemedDescription.Should().Be("Converted archer");
        wildcardLookup.StatModifiers.Should().BeSameAs(dictionary.Patterns[0].StatModifiers);

        dictionary.LookupByThemedId("CIS_SCOUT").Should().Be("enemy_scout");
        dictionary.LookupByThemedId("cis_archer").Should().Be("enemy_archer");

        CrosswalkDictionary.MatchesPattern("enemy_archer", "enemy_*").Should().BeTrue();
        CrosswalkDictionary.MatchesPattern("enemy_archer", string.Empty).Should().BeFalse();
        CrosswalkDictionary.ExtractWildcardValue("enemy_archer", "enemy_*").Should().Be("archer");
        CrosswalkDictionary.ReverseApplyPattern("cis_archer", dictionary.Patterns[0]).Should().Be("enemy_archer");

        IReadOnlyList<CrosswalkEntry> allEntries = dictionary.GetAllEntries();
        allEntries.Should().ContainSingle().Which.Should().BeSameAs(exactEntry);
    }
}
