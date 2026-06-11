#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using DINOForge.Runtime.Bridge;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Validates that the shipping warfare packs only use <c>vanilla_mapping</c> values
/// recognized by the runtime swap pipeline.
/// </summary>
public class VanillaMappingValidationTests
{
    private static readonly Regex UnitStartRegex = new(
        @"^\s*-\s*id:\s*(?<value>\S+)\s*$",
        RegexOptions.Compiled | RegexOptions.CultureInvariant);

    private static readonly Regex VanillaMappingRegex = new(
        @"^\s*vanilla_mapping:\s*(?<value>\S+)\s*$",
        RegexOptions.Compiled | RegexOptions.CultureInvariant);

    private static readonly string RepoRoot = Path.GetFullPath(
        Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", ".."));

    private static readonly string[] ShippingUnitFiles =
    {
        Path.Combine(RepoRoot, "packs", "warfare-naval", "units", "naval_units.yaml"),
        Path.Combine(RepoRoot, "packs", "warfare-aerial", "units", "aerial_units.yaml")
    };

    [Fact]
    public void ShippingPackVanillaMappings_AreAllRecognized()
    {
        HashSet<string> recognizedMappings = new(
            PackStatMappings.VanillaMappingToComponentType.Keys,
            StringComparer.OrdinalIgnoreCase);

        List<string> unknownMappings = new();

        foreach (string unitFile in ShippingUnitFiles)
        {
            string packName = Path.GetFileName(Path.GetDirectoryName(Path.GetDirectoryName(unitFile))!);
            string currentUnitId = "<unknown>";

            foreach (string line in File.ReadLines(unitFile))
            {
                Match unitMatch = UnitStartRegex.Match(line);
                if (unitMatch.Success)
                {
                    currentUnitId = unitMatch.Groups["value"].Value;
                    continue;
                }

                Match mappingMatch = VanillaMappingRegex.Match(line);
                if (!mappingMatch.Success)
                    continue;

                string vanillaMapping = mappingMatch.Groups["value"].Value;
                if (PackStatMappings.TryResolveMapping(vanillaMapping, out _))
                    continue;

                unknownMappings.Add(
                    $"{packName}:{Path.GetFileName(unitFile)} unit={currentUnitId} mapping={vanillaMapping}");
            }
        }

        unknownMappings.Should().BeEmpty(
            "all shipping-pack vanilla_mapping values must be recognized by the runtime swap pipeline. " +
            "Known mappings: {0}. Unknown entries: {1}",
            string.Join(", ", recognizedMappings.OrderBy(value => value)),
            string.Join("; ", unknownMappings));
    }
}
