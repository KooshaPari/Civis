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

    private static readonly Regex BuildingStartRegex = new(
        @"^\s*-\s*id:\s*(?<value>\S+)\s*$",
        RegexOptions.Compiled | RegexOptions.CultureInvariant);

    private static readonly Regex VanillaMappingRegex = new(
        @"^\s*vanilla_mapping:\s*(?<value>\S+)\s*$",
        RegexOptions.Compiled | RegexOptions.CultureInvariant);

    private static readonly string RepoRoot = Path.GetFullPath(
        Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", ".."));

    private static readonly string[] ShippingUnitFiles = BuildShippingUnitFiles();
    private static readonly string[] ShippingBuildingFiles = BuildShippingBuildingFiles();

    private static string[] BuildShippingUnitFiles()
    {
        List<string> unitFiles =
        [
            Path.Combine(RepoRoot, "packs", "warfare-naval", "units", "naval_units.yaml"),
            Path.Combine(RepoRoot, "packs", "warfare-aerial", "units", "aerial_units.yaml"),
            Path.Combine(RepoRoot, "packs", "warfare-starwars", "units", "cims.yaml"),
            Path.Combine(RepoRoot, "packs", "warfare-starwars", "units", "cis_units.yaml"),
            Path.Combine(RepoRoot, "packs", "warfare-starwars", "units", "republic_units.yaml")
        ];

        string modernUnitsDirectory = Path.Combine(RepoRoot, "packs", "warfare-modern", "units");
        if (Directory.Exists(modernUnitsDirectory))
            unitFiles.AddRange(Directory.GetFiles(modernUnitsDirectory, "*.yaml"));

        return unitFiles.ToArray();
    }

    private static string[] BuildShippingBuildingFiles()
    {
        List<string> buildingFiles = [];

        string packsDirectory = Path.Combine(RepoRoot, "packs");
        if (!Directory.Exists(packsDirectory))
            return buildingFiles.ToArray();

        foreach (string packDirectory in Directory.GetDirectories(packsDirectory))
        {
            string buildingsDirectory = Path.Combine(packDirectory, "buildings");
            if (!Directory.Exists(buildingsDirectory))
                continue;

            buildingFiles.AddRange(Directory.GetFiles(buildingsDirectory, "*.yaml"));
        }

        return buildingFiles.ToArray();
    }

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

    [Fact]
    public void ShippingPackBuildingVanillaMappings_AreAllRecognized()
    {
        HashSet<string> recognizedMappings = new(
            PackStatMappings.VanillaMappingToComponentType.Keys,
            StringComparer.OrdinalIgnoreCase);

        List<string> unknownMappings = new();

        foreach (string buildingFile in ShippingBuildingFiles)
        {
            string packName = Path.GetFileName(Path.GetDirectoryName(Path.GetDirectoryName(buildingFile))!);
            string currentBuildingId = "<unknown>";

            foreach (string line in File.ReadLines(buildingFile))
            {
                Match buildingMatch = BuildingStartRegex.Match(line);
                if (buildingMatch.Success)
                {
                    currentBuildingId = buildingMatch.Groups["value"].Value;
                    continue;
                }

                Match mappingMatch = VanillaMappingRegex.Match(line);
                if (!mappingMatch.Success)
                    continue;

                string vanillaMapping = mappingMatch.Groups["value"].Value;
                if (PackStatMappings.TryResolveMapping(vanillaMapping, out _))
                    continue;

                unknownMappings.Add(
                    $"{packName}:{Path.GetFileName(buildingFile)} building={currentBuildingId} mapping={vanillaMapping}");
            }
        }

        unknownMappings.Should().BeEmpty(
            "all shipping-pack building vanilla_mapping values must be recognized by the runtime swap pipeline. " +
            "Known mappings: {0}. Unknown entries: {1}",
            string.Join(", ", recognizedMappings.OrderBy(value => value)),
            string.Join("; ", unknownMappings));
    }
}
