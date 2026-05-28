#nullable enable
using System.CommandLine;
using System.Text.Json;
using System.Text.Json.Serialization;
using Spectre.Console;
using YamlDotNet.RepresentationModel;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Compares two packs and shows overlap/conflicts in units, buildings, factions, and weapons.
/// </summary>
internal static class PackDiffCommand
{
    /// <summary>
    /// Creates the <c>pack diff</c> subcommand.
    /// </summary>
    public static Command Create()
    {
        Argument<string> packAArg = new("packA") { Description = "First pack ID (e.g., warfare-starwars)" };
        Argument<string> packBArg = new("packB") { Description = "Second pack ID (e.g., warfare-modern)" };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();
        Option<bool> showStatsOpt = new("--show-stats") { Description = "Show stat-level diffs for entities in both packs" };

        Command command = new("diff", "Compare two packs and show overlap/conflicts")
        {
            packAArg,
            packBArg,
            formatOpt,
            showStatsOpt
        };

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);
            bool showStats = parseResult.GetValue(showStatsOpt);
            string packA = parseResult.GetRequiredValue(packAArg);
            string packB = parseResult.GetRequiredValue(packBArg);

            try
            {
                var diff = await ComparePacks(packA, packB, showStats, ct).ConfigureAwait(false);

                if (json)
                {
                    CommandOutput.WriteJson(diff);
                }
                else
                {
                    DisplayDiffAsTable(diff);
                }
            }
            catch (Exception ex)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("pack_diff_error", ex.Message);
                }
                else
                {
                    AnsiConsole.MarkupLine($"[red]✗ Error:[/] {Markup.Escape(ex.Message)}");
                }

                Environment.Exit(1);
            }
        });

        return command;
    }

    /// <summary>
    /// Compares two packs by reading their manifests and content YAML files.
    /// </summary>
    private static async Task<PackDiffResult> ComparePacks(
        string packA,
        string packB,
        bool showStats,
        CancellationToken ct)
    {
        var repoRoot = FindRepositoryRoot();
        var packPathA = Path.Combine(repoRoot, "packs", packA);
        var packPathB = Path.Combine(repoRoot, "packs", packB);

        if (!Directory.Exists(packPathA))
            throw new InvalidOperationException($"Pack not found: {packPathA}");
        if (!Directory.Exists(packPathB))
            throw new InvalidOperationException($"Pack not found: {packPathB}");

        var manifestA = await LoadPackManifest(packPathA, ct).ConfigureAwait(false);
        var manifestB = await LoadPackManifest(packPathB, ct).ConfigureAwait(false);

        var contentA = await LoadPackContent(packPathA, manifestA, ct).ConfigureAwait(false);
        var contentB = await LoadPackContent(packPathB, manifestB, ct).ConfigureAwait(false);

        return ComputeDiff(packA, packB, contentA, contentB, showStats);
    }

    /// <summary>
    /// Loads the pack.yaml manifest file.
    /// </summary>
    private static async Task<Dictionary<string, object?>> LoadPackManifest(
        string packPath,
        CancellationToken ct)
    {
        var manifestPath = Path.Combine(packPath, "pack.yaml");
        if (!File.Exists(manifestPath))
            throw new InvalidOperationException($"pack.yaml not found in {packPath}");

        string yaml = await File.ReadAllTextAsync(manifestPath, ct).ConfigureAwait(false);

        var input = new StringReader(yaml);
        var yamlStream = new YamlStream();
        yamlStream.Load(input);

        if (yamlStream.Documents.Count == 0)
            throw new InvalidOperationException($"No YAML document found in {manifestPath}");

        var root = (YamlMappingNode)yamlStream.Documents[0].RootNode;
        var result = new Dictionary<string, object?>();

        foreach (var entry in root.Children)
        {
            var key = ((YamlScalarNode)entry.Key).Value;
            var value = entry.Value;

            if (value is YamlMappingNode mappingValue)
            {
                result[key] = ConvertYamlMapping(mappingValue);
            }
            else if (value is YamlSequenceNode seqValue)
            {
                result[key] = ConvertYamlSequence(seqValue);
            }
            else if (value is YamlScalarNode scalarValue)
            {
                result[key] = scalarValue.Value;
            }
        }

        return result;
    }

    /// <summary>
    /// Loads pack content (units, buildings, factions, weapons, etc.) from YAML files referenced in the manifest.
    /// </summary>
    private static async Task<PackContentData> LoadPackContent(
        string packPath,
        Dictionary<string, object?> manifest,
        CancellationToken ct)
    {
        var content = new PackContentData();

        if (manifest.TryGetValue("loads", out var loadsObj) && loadsObj is Dictionary<string, object?> loads)
        {
            // Load units
            if (loads.TryGetValue("units", out var unitsObj) && unitsObj is List<object?> unitsList)
            {
                foreach (var unitFile in unitsList.OfType<string>())
                {
                    var filePath = Path.Combine(packPath, unitFile + ".yaml");
                    if (File.Exists(filePath))
                    {
                        var units = await LoadYamlListFile(filePath, ct).ConfigureAwait(false);
                        foreach (var unit in units)
                        {
                            if (unit is Dictionary<string, object?> unitDict && unitDict.TryGetValue("id", out var idObj))
                            {
                                content.Units[idObj?.ToString() ?? "unknown"] = unitDict;
                            }
                        }
                    }
                }
            }

            // Load buildings
            if (loads.TryGetValue("buildings", out var buildingsObj) && buildingsObj is List<object?> buildingsList)
            {
                foreach (var buildingFile in buildingsList.OfType<string>())
                {
                    var filePath = Path.Combine(packPath, buildingFile + ".yaml");
                    if (File.Exists(filePath))
                    {
                        var buildings = await LoadYamlListFile(filePath, ct).ConfigureAwait(false);
                        foreach (var building in buildings)
                        {
                            if (building is Dictionary<string, object?> buildingDict && buildingDict.TryGetValue("id", out var idObj))
                            {
                                content.Buildings[idObj?.ToString() ?? "unknown"] = buildingDict;
                            }
                        }
                    }
                }
            }

            // Load factions
            if (loads.TryGetValue("factions", out var factionsObj) && factionsObj is List<object?> factionsList)
            {
                foreach (var factionFile in factionsList.OfType<string>())
                {
                    var filePath = Path.Combine(packPath, factionFile + ".yaml");
                    if (File.Exists(filePath))
                    {
                        var factions = await LoadYamlListFile(filePath, ct).ConfigureAwait(false);
                        foreach (var faction in factions)
                        {
                            if (faction is Dictionary<string, object?> factionDict && factionDict.TryGetValue("id", out var idObj))
                            {
                                content.Factions[idObj?.ToString() ?? "unknown"] = factionDict;
                            }
                        }
                    }
                }
            }

            // Load weapons
            if (loads.TryGetValue("weapons", out var weaponsObj) && weaponsObj is List<object?> weaponsList)
            {
                foreach (var weaponFile in weaponsList.OfType<string>())
                {
                    var filePath = Path.Combine(packPath, weaponFile + ".yaml");
                    if (File.Exists(filePath))
                    {
                        var weapons = await LoadYamlListFile(filePath, ct).ConfigureAwait(false);
                        foreach (var weapon in weapons)
                        {
                            if (weapon is Dictionary<string, object?> weaponDict && weaponDict.TryGetValue("id", out var idObj))
                            {
                                content.Weapons[idObj?.ToString() ?? "unknown"] = weaponDict;
                            }
                        }
                    }
                }
            }

            // Load doctrines
            if (loads.TryGetValue("doctrines", out var doctrinesObj) && doctrinesObj is List<object?> doctrinesList)
            {
                foreach (var doctrineFile in doctrinesList.OfType<string>())
                {
                    var filePath = Path.Combine(packPath, doctrineFile + ".yaml");
                    if (File.Exists(filePath))
                    {
                        var doctrines = await LoadYamlListFile(filePath, ct).ConfigureAwait(false);
                        foreach (var doctrine in doctrines)
                        {
                            if (doctrine is Dictionary<string, object?> doctrineDict && doctrineDict.TryGetValue("id", out var idObj))
                            {
                                content.Doctrines[idObj?.ToString() ?? "unknown"] = doctrineDict;
                            }
                        }
                    }
                }
            }
        }

        return content;
    }

    /// <summary>
    /// Loads a YAML file as a list of objects.
    /// </summary>
    private static async Task<List<object?>> LoadYamlListFile(
        string filePath,
        CancellationToken ct)
    {
        string yaml = await File.ReadAllTextAsync(filePath, ct).ConfigureAwait(false);

        var input = new StringReader(yaml);
        var yamlStream = new YamlStream();
        yamlStream.Load(input);

        var result = new List<object?>();

        if (yamlStream.Documents.Count > 0)
        {
            var root = yamlStream.Documents[0].RootNode;

            if (root is YamlSequenceNode seq)
            {
                foreach (var item in seq.Children)
                {
                    result.Add(ConvertYamlNode(item));
                }
            }
        }

        return result;
    }

    /// <summary>
    /// Converts a YAML node to a C# object.
    /// </summary>
    private static object? ConvertYamlNode(YamlNode node)
    {
        return node switch
        {
            YamlScalarNode scalar => scalar.Value,
            YamlMappingNode mapping => ConvertYamlMapping(mapping),
            YamlSequenceNode sequence => ConvertYamlSequence(sequence),
            _ => null
        };
    }

    /// <summary>
    /// Converts a YAML mapping to a dictionary.
    /// </summary>
    private static Dictionary<string, object?> ConvertYamlMapping(YamlMappingNode mapping)
    {
        var result = new Dictionary<string, object?>();

        foreach (var entry in mapping.Children)
        {
            var key = ((YamlScalarNode)entry.Key).Value;
            result[key] = ConvertYamlNode(entry.Value);
        }

        return result;
    }

    /// <summary>
    /// Converts a YAML sequence to a list.
    /// </summary>
    private static List<object?> ConvertYamlSequence(YamlSequenceNode sequence)
    {
        var result = new List<object?>();

        foreach (var item in sequence.Children)
        {
            result.Add(ConvertYamlNode(item));
        }

        return result;
    }

    /// <summary>
    /// Computes the diff result by comparing two pack contents.
    /// </summary>
    private static PackDiffResult ComputeDiff(
        string packA,
        string packB,
        PackContentData contentA,
        PackContentData contentB,
        bool showStats)
    {
        var result = new PackDiffResult { PackA = packA, PackB = packB };

        // Compare units
        result.Units = CompareDictionaries(contentA.Units, contentB.Units, showStats);

        // Compare buildings
        result.Buildings = CompareDictionaries(contentA.Buildings, contentB.Buildings, showStats);

        // Compare factions
        result.Factions = CompareDictionaries(contentA.Factions, contentB.Factions, showStats);

        // Compare weapons
        result.Weapons = CompareDictionaries(contentA.Weapons, contentB.Weapons, showStats);

        // Compare doctrines
        result.Doctrines = CompareDictionaries(contentA.Doctrines, contentB.Doctrines, showStats);

        return result;
    }

    /// <summary>
    /// Compares two dictionaries of entities and returns categorized lists.
    /// </summary>
    private static EntityComparisonResult CompareDictionaries(
        Dictionary<string, Dictionary<string, object?>> dictA,
        Dictionary<string, Dictionary<string, object?>> dictB,
        bool showStats)
    {
        var result = new EntityComparisonResult();

        // Entities in A only
        foreach (var (id, entity) in dictA)
        {
            if (!dictB.ContainsKey(id))
            {
                result.OnlyInA.Add(id);
            }
        }

        // Entities in B only
        foreach (var (id, entity) in dictB)
        {
            if (!dictA.ContainsKey(id))
            {
                result.OnlyInB.Add(id);
            }
        }

        // Entities in both
        foreach (var (id, entityA) in dictA)
        {
            if (dictB.TryGetValue(id, out var entityB))
            {
                result.InBoth.Add(id);

                if (showStats)
                {
                    var diff = ComputeEntityDiff(entityA, entityB);
                    if (diff.Count > 0)
                    {
                        result.StatDiffs[id] = diff;
                    }
                }
            }
        }

        return result;
    }

    /// <summary>
    /// Computes stat-level differences between two entities.
    /// </summary>
    private static Dictionary<string, (object? A, object? B)> ComputeEntityDiff(
        Dictionary<string, object?> entityA,
        Dictionary<string, object?> entityB)
    {
        var result = new Dictionary<string, (object?, object?)>();

        // Check all keys in A
        foreach (var (key, valueA) in entityA)
        {
            if (entityB.TryGetValue(key, out var valueB))
            {
                if (!Equals(valueA, valueB))
                {
                    result[key] = (valueA, valueB);
                }
            }
            else
            {
                result[key] = (valueA, null);
            }
        }

        // Check for keys in B but not A
        foreach (var (key, valueB) in entityB)
        {
            if (!entityA.ContainsKey(key))
            {
                result[key] = (null, valueB);
            }
        }

        return result;
    }

    /// <summary>
    /// Displays the diff result as a formatted Spectre.Console table.
    /// </summary>
    private static void DisplayDiffAsTable(PackDiffResult diff)
    {
        AnsiConsole.MarkupLine($"\n[bold cyan]Pack Diff: {Markup.Escape(diff.PackA)} vs {Markup.Escape(diff.PackB)}[/]\n");

        DisplayCategoryDiff("Units", diff.Units);
        DisplayCategoryDiff("Buildings", diff.Buildings);
        DisplayCategoryDiff("Factions", diff.Factions);
        DisplayCategoryDiff("Weapons", diff.Weapons);
        DisplayCategoryDiff("Doctrines", diff.Doctrines);
    }

    /// <summary>
    /// Displays one category of the diff (units, buildings, etc.) as a table.
    /// </summary>
    private static void DisplayCategoryDiff(string category, EntityComparisonResult comparison)
    {
        if (comparison.OnlyInA.Count == 0 && comparison.OnlyInB.Count == 0 && comparison.InBoth.Count == 0)
            return;

        var table = new Table()
            .Border(TableBorder.Rounded)
            .Title($"[bold]{category}[/]")
            .AddColumn("[green]In A Only[/]")
            .AddColumn("[blue]In B Only[/]")
            .AddColumn("[yellow]In Both[/]");

        int maxRows = Math.Max(comparison.OnlyInA.Count, Math.Max(comparison.OnlyInB.Count, comparison.InBoth.Count));

        for (int i = 0; i < maxRows; i++)
        {
            var rowA = i < comparison.OnlyInA.Count ? Markup.Escape(comparison.OnlyInA[i]) : "";
            var rowB = i < comparison.OnlyInB.Count ? Markup.Escape(comparison.OnlyInB[i]) : "";
            var rowBoth = i < comparison.InBoth.Count ? Markup.Escape(comparison.InBoth[i]) : "";

            table.AddRow(rowA, rowB, rowBoth);
        }

        AnsiConsole.Write(table);

        // Display stat diffs if available
        if (comparison.StatDiffs.Count > 0)
        {
            AnsiConsole.MarkupLine($"\n[bold yellow]Stat Differences in {category}:[/]");
            foreach (var (entityId, diffs) in comparison.StatDiffs)
            {
                AnsiConsole.MarkupLine($"  [cyan]{Markup.Escape(entityId)}[/]:");
                foreach (var (key, (valA, valB)) in diffs)
                {
                    AnsiConsole.MarkupLine($"    {Markup.Escape(key)}: {Markup.Escape(valA?.ToString() ?? "null")} → {Markup.Escape(valB?.ToString() ?? "null")}");
                }
            }
        }

        AnsiConsole.WriteLine();
    }

    /// <summary>
    /// Finds the repository root directory by looking for the .git folder.
    /// </summary>
    private static string FindRepositoryRoot()
    {
        var current = Directory.GetCurrentDirectory();

        while (true)
        {
            if (Directory.Exists(Path.Combine(current, ".git")))
                return current;

            var parent = Directory.GetParent(current);
            if (parent is null)
                throw new InvalidOperationException("Could not find repository root (.git directory)");

            current = parent.FullName;
        }
    }

    /// <summary>
    /// Result data structure for pack diff.
    /// </summary>
    [Serializable]
    public class PackDiffResult
    {
        [JsonPropertyName("packA")]
        public string PackA { get; set; } = string.Empty;

        [JsonPropertyName("packB")]
        public string PackB { get; set; } = string.Empty;

        [JsonPropertyName("units")]
        public EntityComparisonResult Units { get; set; } = new();

        [JsonPropertyName("buildings")]
        public EntityComparisonResult Buildings { get; set; } = new();

        [JsonPropertyName("factions")]
        public EntityComparisonResult Factions { get; set; } = new();

        [JsonPropertyName("weapons")]
        public EntityComparisonResult Weapons { get; set; } = new();

        [JsonPropertyName("doctrines")]
        public EntityComparisonResult Doctrines { get; set; } = new();
    }

    /// <summary>
    /// Result structure for comparing a single entity category.
    /// </summary>
    [Serializable]
    public class EntityComparisonResult
    {
        [JsonPropertyName("onlyInA")]
        public List<string> OnlyInA { get; set; } = new();

        [JsonPropertyName("onlyInB")]
        public List<string> OnlyInB { get; set; } = new();

        [JsonPropertyName("inBoth")]
        public List<string> InBoth { get; set; } = new();

        [JsonPropertyName("statDiffs")]
        public Dictionary<string, Dictionary<string, (object?, object?)>> StatDiffs { get; set; } = new();
    }

    /// <summary>
    /// Internal structure to hold loaded pack content.
    /// </summary>
    private class PackContentData
    {
        public Dictionary<string, Dictionary<string, object?>> Units { get; } = new();
        public Dictionary<string, Dictionary<string, object?>> Buildings { get; } = new();
        public Dictionary<string, Dictionary<string, object?>> Factions { get; } = new();
        public Dictionary<string, Dictionary<string, object?>> Weapons { get; } = new();
        public Dictionary<string, Dictionary<string, object?>> Doctrines { get; } = new();
    }
}
