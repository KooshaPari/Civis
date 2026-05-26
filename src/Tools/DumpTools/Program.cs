using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Spectre.Console;

namespace DINOForge.Tools.DumpTools
{
    /// <summary>
    /// CLI tool for analyzing entity dumps produced by the DINOForge runtime plugin.
    /// Usage:
    ///   dotnet run -- analyze &lt;dump-dir&gt;     Analyze a dump directory
    ///   dotnet run -- list                       List available dumps
    ///   dotnet run -- components &lt;dump-dir&gt;  Show all ECS component types
    ///   dotnet run -- systems &lt;dump-dir&gt;     Show all ECS systems
    ///   dotnet run -- namespaces &lt;dump-dir&gt;  Show game namespace map
    /// </summary>
    class Program
    {
        static int Main(string[] args)
        {
            if (args.Length == 0)
            {
                PrintHelp();
                return 0;
            }

            string command = args[0].ToLower();

            try
            {
                return command switch
                {
                    "list" => ListDumps(),
                    "analyze" => AnalyzeDump(args.ElementAtOrDefault(1)),
                    "components" => ShowComponents(args.ElementAtOrDefault(1)),
                    "systems" => ShowSystems(args.ElementAtOrDefault(1)),
                    "namespaces" => ShowNamespaces(args.ElementAtOrDefault(1)),
                    "help" or "--help" or "-h" => PrintHelp(),
                    _ => PrintHelp()
                };
            }
            catch (Exception ex)
            {
                AnsiConsole.WriteException(ex);
                return 1;
            }
        }

        static int PrintHelp()
        {
            AnsiConsole.MarkupLine("[bold]DINOForge Dump Analyzer[/]");
            AnsiConsole.MarkupLine("");
            AnsiConsole.MarkupLine("Commands:");
            AnsiConsole.MarkupLine("  [green]list[/]                    List available dumps");
            AnsiConsole.MarkupLine("  [green]analyze[/] <dump-dir>      Full dump analysis");
            AnsiConsole.MarkupLine("  [green]components[/] <dump-dir>   Show ECS component types");
            AnsiConsole.MarkupLine("  [green]systems[/] <dump-dir>      Show ECS systems");
            AnsiConsole.MarkupLine("  [green]namespaces[/] <dump-dir>   Show game namespaces");
            return 0;
        }

        static string GetDumpsRoot()
        {
            // Default to BepInEx dumps location
            string? bepInExRoot = Environment.GetEnvironmentVariable("BEPINEX_ROOT");
            if (bepInExRoot != null)
                return Path.Combine(bepInExRoot, "dinoforge_dumps");

            // Fallback: look relative to game install (configurable via DINOFORGE_GAME_PATH)
            string gamePath = Environment.GetEnvironmentVariable("DINOFORGE_GAME_PATH")
                ?? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles), "Steam", "steamapps", "common", "Diplomacy is Not an Option");
            return Path.Combine(gamePath, "BepInEx", "dinoforge_dumps");
        }

        static int ListDumps()
        {
            string root = GetDumpsRoot();
            if (!Directory.Exists(root))
            {
                AnsiConsole.MarkupLine($"[red]Dumps directory not found: {root}[/]");
                AnsiConsole.MarkupLine("Run the game with DINOForge to generate dumps.");
                return 1;
            }

            string[] dumps = Directory.GetDirectories(root)
                .OrderByDescending(d => d)
                .ToArray();

            Table table = new Table();
            table.AddColumn("Dump");
            table.AddColumn("Date");
            table.AddColumn("Files");

            foreach (string dump in dumps)
            {
                string name = Path.GetFileName(dump);
                int fileCount = Directory.GetFiles(dump, "*.json", SearchOption.AllDirectories).Length;
                table.AddRow(name, Directory.GetCreationTime(dump).ToString("yyyy-MM-dd HH:mm"), fileCount.ToString());
            }

            AnsiConsole.Write(table);
            return 0;
        }

        static int AnalyzeDump(string? dumpDir)
        {
            dumpDir = ResolveDumpDir(dumpDir);
            if (dumpDir == null) return 1;

            AnsiConsole.MarkupLine($"[bold]Analyzing: {dumpDir}[/]");

            // Show worlds
            string worldsPath = Path.Combine(dumpDir, "worlds.json");
            if (File.Exists(worldsPath))
            {
                JArray worlds = JArray.Parse(File.ReadAllText(worldsPath, Encoding.UTF8));
                AnsiConsole.MarkupLine($"\n[bold]Worlds: {worlds.Count}[/]");
                foreach (JToken world in worlds)
                {
                    AnsiConsole.MarkupLine($"  {world["name"]}: {world["entityCount"]} entities");
                }
            }

            // Show component counts
            ShowComponents(dumpDir);

            // Show system counts
            ShowSystems(dumpDir);

            return 0;
        }

        static int ShowComponents(string? dumpDir)
        {
            dumpDir = ResolveDumpDir(dumpDir);
            if (dumpDir == null) return 1;

            string typesPath = Path.Combine(dumpDir, "ecs_types.json");
            if (!File.Exists(typesPath))
            {
                AnsiConsole.MarkupLine("[red]ecs_types.json not found[/]");
                return 1;
            }

            JArray types = JArray.Parse(File.ReadAllText(typesPath, Encoding.UTF8));

            // Group by category
            var components = types.Where(t => (bool)(t["isComponent"] ?? false)).ToList();
            var sharedComponents = types.Where(t => (bool)(t["isSharedComponent"] ?? false)).ToList();
            var buffers = types.Where(t => (bool)(t["isBuffer"] ?? false)).ToList();
            var systems = types.Where(t => (bool)(t["isSystem"] ?? false)).ToList();

            AnsiConsole.MarkupLine($"\n[bold]ECS Types Summary[/]");
            AnsiConsole.MarkupLine($"  Components: {components.Count}");
            AnsiConsole.MarkupLine($"  Shared Components: {sharedComponents.Count}");
            AnsiConsole.MarkupLine($"  Buffers: {buffers.Count}");
            AnsiConsole.MarkupLine($"  Systems: {systems.Count}");

            // Group by assembly
            var byAssembly = types.GroupBy(t => t["assembly"]?.ToString() ?? "Unknown")
                .OrderByDescending(g => g.Count());

            Table table = new Table();
            table.AddColumn("Assembly");
            table.AddColumn("Types");

            foreach (var group in byAssembly)
            {
                table.AddRow(group.Key, group.Count().ToString());
            }

            AnsiConsole.Write(table);

            // Show DNO.Main types specifically
            var dnoTypes = types.Where(t => (t["assembly"]?.ToString() ?? "") == "DNO.Main").ToList();
            if (dnoTypes.Any())
            {
                AnsiConsole.MarkupLine($"\n[bold]DNO.Main ECS Types ({dnoTypes.Count}):[/]");

                Table dnoTable = new Table();
                dnoTable.AddColumn("Type");
                dnoTable.AddColumn("Namespace");
                dnoTable.AddColumn("Kind");
                dnoTable.AddColumn("Fields");

                foreach (JToken type in dnoTypes.OrderBy(t => t["fullName"]?.ToString()))
                {
                    string kind = "";
                    if ((bool)(type["isSystem"] ?? false)) kind = "System";
                    else if ((bool)(type["isComponent"] ?? false)) kind = "Component";
                    else if ((bool)(type["isSharedComponent"] ?? false)) kind = "SharedComp";
                    else if ((bool)(type["isBuffer"] ?? false)) kind = "Buffer";

                    int fieldCount = (type["fields"] as JArray)?.Count ?? 0;

                    dnoTable.AddRow(
                        type["name"]?.ToString() ?? "?",
                        type["namespace"]?.ToString() ?? "?",
                        kind,
                        fieldCount.ToString());
                }

                AnsiConsole.Write(dnoTable);
            }

            return 0;
        }

        static int ShowSystems(string? dumpDir)
        {
            dumpDir = ResolveDumpDir(dumpDir);
            if (dumpDir == null) return 1;

            string[] systemFiles = Directory.GetFiles(dumpDir, "systems_*.json");
            if (systemFiles.Length == 0)
            {
                AnsiConsole.MarkupLine("[yellow]No system dump files found[/]");
                return 0;
            }

            foreach (string file in systemFiles)
            {
                string worldName = Path.GetFileNameWithoutExtension(file).Replace("systems_", "");
                JArray systems = JArray.Parse(File.ReadAllText(file, Encoding.UTF8));

                AnsiConsole.MarkupLine($"\n[bold]Systems in '{worldName}' ({systems.Count}):[/]");

                Table table = new Table();
                table.AddColumn("System");
                table.AddColumn("Namespace");
                table.AddColumn("Group");
                table.AddColumn("Enabled");

                foreach (JToken sys in systems.OrderBy(s => s["namespace"]?.ToString()))
                {
                    table.AddRow(
                        sys["name"]?.ToString() ?? "?",
                        sys["namespace"]?.ToString() ?? "?",
                        sys["updateGroup"]?.ToString() ?? "-",
                        (bool)(sys["enabled"] ?? true) ? "Yes" : "No");
                }

                AnsiConsole.Write(table);
            }

            return 0;
        }

        static int ShowNamespaces(string? dumpDir)
        {
            dumpDir = ResolveDumpDir(dumpDir);
            if (dumpDir == null) return 1;

            string nsPath = Path.Combine(dumpDir, "game_namespaces.json");
            if (!File.Exists(nsPath))
            {
                AnsiConsole.MarkupLine("[red]game_namespaces.json not found[/]");
                return 1;
            }

            JObject namespaces = JObject.Parse(File.ReadAllText(nsPath, Encoding.UTF8));

            foreach (var assembly in namespaces.Properties())
            {
                AnsiConsole.MarkupLine($"\n[bold]{assembly.Name}[/]");

                if (assembly.Value is JObject nsObj)
                {
                    Tree tree = new Tree(assembly.Name);

                    foreach (var ns in nsObj.Properties().OrderBy(p => p.Name))
                    {
                        int typeCount = (ns.Value as JArray)?.Count ?? 0;
                        tree.AddNode($"{ns.Name} ({typeCount} types)");
                    }

                    AnsiConsole.Write(tree);
                }
            }

            return 0;
        }

        static string? ResolveDumpDir(string? dumpDir)
        {
            if (string.IsNullOrEmpty(dumpDir))
            {
                // Use latest dump
                string root = GetDumpsRoot();
                if (!Directory.Exists(root))
                {
                    AnsiConsole.MarkupLine($"[red]Dumps directory not found: {root}[/]");
                    return null;
                }

                dumpDir = Directory.GetDirectories(root)
                    .OrderByDescending(d => d)
                    .FirstOrDefault();

                if (dumpDir == null)
                {
                    AnsiConsole.MarkupLine("[red]No dumps found. Run the game with DINOForge first.[/]");
                    return null;
                }

                AnsiConsole.MarkupLine($"[grey]Using latest dump: {Path.GetFileName(dumpDir)}[/]");
            }

            if (!Directory.Exists(dumpDir))
            {
                AnsiConsole.MarkupLine($"[red]Directory not found: {dumpDir}[/]");
                return null;
            }

            return dumpDir;
        }
    }
}
