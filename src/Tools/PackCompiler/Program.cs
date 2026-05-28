using System;
using System.Collections.Generic;
using System.CommandLine;
using System.Diagnostics;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using System.Security.Cryptography;
using Spectre.Console;
using DINOForge.SDK;
using DINOForge.SDK.Assets;
using DINOForge.SDK.IO;
using DINOForge.SDK.Models;
using DINOForge.SDK.Signing;
using DINOForge.SDK.Validation;
using DINOForge.Tools.PackCompiler.Json;
using DINOForge.Tools.PackCompiler.Models;
using DINOForge.Tools.PackCompiler.Services;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;
using System.Diagnostics.CodeAnalysis;

namespace DINOForge.Tools.PackCompiler
{
    class Program
    {
        [System.Diagnostics.CodeAnalysis.UnconditionalSuppressMessage("Trimming", "IL2026", Justification = "PackCompiler command handlers intentionally invoke trim-sensitive serialization helpers in a CLI entrypoint.")]
        static async Task<int> Main(string[] args)
        {
            var packPathArg = new Argument<string>("pack-path") { Description = "Path to the pack directory" };
            var outputOption = new Option<string?>("--output", "-o") { Description = "Output directory for the bundled pack" };
            var formatOption = new Option<string>("--format") { Description = "Output format: text or json (default: text)", DefaultValueFactory = _ => "text" };

            // Validate command
            var validateCommand = new Command("validate") { Description = "Validate a pack directory" };
            validateCommand.Arguments.Add(packPathArg);
            validateCommand.Options.Add(formatOption);
            validateCommand.SetAction(parseResult =>
            {
                string packPath = parseResult.GetValue(packPathArg)!;
                string format = parseResult.GetValue(formatOption) ?? "text";
                ValidatePack(packPath, format);
            });

            // Build command
            var buildPackPathArg = new Argument<string>("pack-path") { Description = "Path to the pack directory" };
            var buildFormatOption = new Option<string>("--format") { Description = "Output format: text or json (default: text)", DefaultValueFactory = _ => "text" };
            var buildCommand = new Command("build") { Description = "Validate and bundle a pack directory" };
            buildCommand.Arguments.Add(buildPackPathArg);
            buildCommand.Options.Add(outputOption);
            buildCommand.Options.Add(buildFormatOption);
            buildCommand.SetAction(parseResult =>
            {
                string packPath = parseResult.GetValue(buildPackPathArg)!;
                string? outputDir = parseResult.GetValue(outputOption);
                string format = parseResult.GetValue(buildFormatOption) ?? "text";
                BuildPack(packPath, outputDir, format);
            });

            // Assets command group (v0.7.0+: unified asset pipeline)
            var assetsCommand = new Command("assets") { Description = "Asset pipeline management: import, validate, optimize, generate" };

            var packPathArgAssets = new Argument<string>("pack-path") { Description = "Path to the pack directory with asset_pipeline.yaml" };

            // Asset pipeline subcommands
            var assetImportCommand = new Command("import") { Description = "Import 3D models (GLB/FBX) from asset_pipeline.yaml" };
            assetImportCommand.Arguments.Add(packPathArgAssets);
            assetImportCommand.SetAction(parseResult =>
            {
                string packPath = parseResult.GetValue(packPathArgAssets)!;
                AssetImport(packPath);
            });

            var assetValidateCommand = new Command("validate") { Description = "Validate imported assets against config" };
            assetValidateCommand.Arguments.Add(packPathArgAssets);
            assetValidateCommand.SetAction(parseResult =>
            {
                string packPath = parseResult.GetValue(packPathArgAssets)!;
                AssetValidate(packPath);
            });

            var assetOptimizeCommand = new Command("optimize") { Description = "Generate LOD variants for assets" };
            assetOptimizeCommand.Arguments.Add(packPathArgAssets);
            assetOptimizeCommand.SetAction(parseResult =>
            {
                string packPath = parseResult.GetValue(packPathArgAssets)!;
                AssetOptimize(packPath);
            });

            var assetGenerateCommand = new Command("generate") { Description = "Generate Unity prefabs from optimized assets" };
            assetGenerateCommand.Arguments.Add(packPathArgAssets);
            assetGenerateCommand.SetAction(parseResult =>
            {
                string packPath = parseResult.GetValue(packPathArgAssets)!;
                AssetGenerate(packPath);
            });

            var assetBuildCommand = new Command("build") { Description = "Run full pipeline: import → validate → optimize → generate" };
            assetBuildCommand.Arguments.Add(packPathArgAssets);
            assetBuildCommand.SetAction(parseResult =>
            {
                string packPath = parseResult.GetValue(packPathArgAssets)!;
                AssetBuild(packPath);
            });

            assetsCommand.Subcommands.Add(assetImportCommand);
            assetsCommand.Subcommands.Add(assetValidateCommand);
            assetsCommand.Subcommands.Add(assetOptimizeCommand);
            assetsCommand.Subcommands.Add(assetGenerateCommand);
            assetsCommand.Subcommands.Add(assetBuildCommand);

            // Bundle inspection commands (kept for backwards compatibility)
            var bundlesCommand = new Command("bundles") { Description = "Inspect and validate Unity asset bundles" };

            // NOTE: Asset bundle inspection commands (list, inspect, validate) require AssetService
            // which is part of the Runtime (BepInEx plugin), not the SDK. These are deprecated.
            // Use the MCP bridge tools instead (game_screenshot, game_analyze_screen, etc.)
            //
            // var gameDirArg = new Argument<string>("game-dir") { Description = "Game installation directory" };
            // var bundlesListCommand = new Command("list") { Description = "List all game asset bundles" };
            // bundlesListCommand.Arguments.Add(gameDirArg);
            // bundlesListCommand.SetAction(parseResult =>
            // {
            //     string gameDir = parseResult.GetValue(gameDirArg)!;
            //     AssetsListBundles(gameDir);
            // });
            //
            // var bundlePathArg = new Argument<string>("bundle-path") { Description = "Path to a .bundle file" };
            // var bundlesInspectCommand = new Command("inspect") { Description = "List assets in a bundle" };
            // bundlesInspectCommand.Arguments.Add(bundlePathArg);
            // bundlesInspectCommand.SetAction(parseResult =>
            // {
            //     string bundlePath = parseResult.GetValue(bundlePathArg)!;
            //     AssetsInspect(bundlePath);
            // });
            //
            // var bundlesValidateCommand = new Command("validate") { Description = "Validate a mod asset bundle" };
            // bundlesValidateCommand.Arguments.Add(bundlePathArg);
            // bundlesValidateCommand.SetAction(parseResult =>
            // {
            //     string bundlePath = parseResult.GetValue(bundlePathArg)!;
            //     AssetsValidate(bundlePath);
            // });
            //
            // bundlesCommand.Subcommands.Add(bundlesListCommand);
            // bundlesCommand.Subcommands.Add(bundlesInspectCommand);
            // bundlesCommand.Subcommands.Add(bundlesValidateCommand);

            // Thunderstore command group
            var thunderstoreCommand = new Command("thunderstore") { Description = "Thunderstore publishing tools" };

            // thunderstore manifest - generate manifest.json
            var tsManifestPackDirArg = new Argument<string>("pack-path") { Description = "Path to the pack directory containing pack.yaml" };
            var tsManifestAuthorOption = new Option<string?>("--author") { Description = "Thunderstore author name (default: from DINOForge config or 'DINOForge')" };
            var tsManifestOutputOption = new Option<string?>("--output", "-o") { Description = "Output directory (defaults to pack-path)" };
            var tsManifestCommand = new Command("manifest") { Description = "Generate Thunderstore-compatible manifest.json" };
            tsManifestCommand.Arguments.Add(tsManifestPackDirArg);
            tsManifestCommand.Options.Add(tsManifestAuthorOption);
            tsManifestCommand.Options.Add(tsManifestOutputOption);
            tsManifestCommand.SetAction(parseResult =>
            {
                string packPath = parseResult.GetValue(tsManifestPackDirArg)!;
                string author = parseResult.GetValue(tsManifestAuthorOption) ?? GetDefaultAuthor();
                string? outputDir = parseResult.GetValue(tsManifestOutputOption);
                GenerateThunderstoreManifest(packPath, author, outputDir ?? "");
            });

            // thunderstore package - create full Thunderstore ZIP
            var tsPackPackDirArg = new Argument<string>("pack-path") { Description = "Path to the pack directory containing pack.yaml" };
            var tsPackAuthorOption = new Option<string?>("--author") { Description = "Thunderstore author name (default: from DINOForge config or 'DINOForge')" };
            var tsPackOutputOption = new Option<string?>("--output", "-o") { Description = "Output directory for ZIP (defaults to dist/)" };
            var tsPackCommand = new Command("package") { Description = "Create Thunderstore-compatible ZIP package" };
            tsPackCommand.Arguments.Add(tsPackPackDirArg);
            tsPackCommand.Options.Add(tsPackAuthorOption);
            tsPackCommand.Options.Add(tsPackOutputOption);
            tsPackCommand.SetAction(parseResult =>
            {
                string packPath = parseResult.GetValue(tsPackPackDirArg)!;
                string author = parseResult.GetValue(tsPackAuthorOption) ?? GetDefaultAuthor();
                string? outputDir = parseResult.GetValue(tsPackOutputOption);
                CreateThunderstorePackage(packPath, author, outputDir ?? "dist");
            });

            thunderstoreCommand.Subcommands.Add(tsManifestCommand);
            thunderstoreCommand.Subcommands.Add(tsPackCommand);

            // Validate total conversion command
            var validateTcCommand = new Command("validate-tc") { Description = "Validate a total conversion pack manifest" };
            var tcManifestArg = new Argument<string>("manifest") { Description = "Path to total-conversion YAML manifest" };
            validateTcCommand.Arguments.Add(tcManifestArg);
            validateTcCommand.SetAction(parseResult =>
            {
                string manifestPath = parseResult.GetValue(tcManifestArg)!;
                ValidateTotalConversion(manifestPath);
            });

            // Pack subcommand - polyrepo pack management
            var packCommand = new Command("pack") { Description = "Manage pack repositories and submodules" };

            // pack add - add a pack repo as a git submodule
            var packAddCommand = new Command("add") { Description = "Add a pack repository as a git submodule" };
            var repoUrlArg = new Argument<string>("repo-url") { Description = "Git repository URL of the pack" };
            var packPathOpt = new Option<string?>("--path") { Description = "Local path for the submodule (defaults to packs/<repo-name>)" };
            packAddCommand.Arguments.Add(repoUrlArg);
            packAddCommand.Options.Add(packPathOpt);
            packAddCommand.SetAction(parseResult =>
            {
                string repoUrl = parseResult.GetValue(repoUrlArg)!;
                string? path = parseResult.GetValue(packPathOpt);
                PackAdd(repoUrl, path ?? "");
            });

            // pack list - list installed pack submodules
            var packListCommand = new Command("list") { Description = "List installed pack submodules" };
            packListCommand.SetAction(_ => PackList());

            // pack update - update all pack submodules
            var packUpdateCommand = new Command("update") { Description = "Update all pack submodules to latest" };
            packUpdateCommand.SetAction(_ => PackUpdate());

            // pack lock - generate packs.lock file
            var packLockCommand = new Command("lock") { Description = "Generate packs.lock for reproducible builds" };
            packLockCommand.SetAction(_ => PackLock());

            packCommand.Subcommands.Add(packAddCommand);
            packCommand.Subcommands.Add(packListCommand);
            packCommand.Subcommands.Add(packUpdateCommand);
            packCommand.Subcommands.Add(packLockCommand);

            // Sign command - cryptographic pack signing
            var signPackPathArg = new Argument<string>("pack-path") { Description = "Path to the pack directory" };
            var signKeyPathOption = new Option<string>("--key", "-k") { Description = "Path to PEM-format RSA private key file (required)" };
            var signOutputOption = new Option<string?>("--output", "-o") { Description = "Output directory for signature file (defaults to pack-path)" };
            var signCommand = new Command("sign") { Description = "Sign a pack with an RSA private key" };
            signCommand.Arguments.Add(signPackPathArg);
            signCommand.Options.Add(signKeyPathOption);
            signCommand.Options.Add(signOutputOption);
            signCommand.SetAction(parseResult =>
            {
                string packPath = parseResult.GetValue(signPackPathArg)!;
                string keyPath = parseResult.GetValue(signKeyPathOption)!;
                string? outputDir = parseResult.GetValue(signOutputOption);
                SignPack(packPath, keyPath, outputDir ?? packPath);
            });

            // Verify command - cryptographic pack verification
            var verifyPackPathArg = new Argument<string>("pack-path") { Description = "Path to the pack directory" };
            var verifyKeysOption = new Option<string?>("--trusted-keys", "-t") { Description = "Path to trusted keys file (optional)" };
            var verifyCommand = new Command("verify") { Description = "Verify a pack signature" };
            verifyCommand.Arguments.Add(verifyPackPathArg);
            verifyCommand.Options.Add(verifyKeysOption);
            verifyCommand.SetAction(parseResult =>
            {
                string packPath = parseResult.GetValue(verifyPackPathArg)!;
                string? trustedKeysFile = parseResult.GetValue(verifyKeysOption);
                VerifyPack(packPath, trustedKeysFile);
            });

            var rootCommand = new RootCommand("DINOForge PackCompiler - Validate and bundle content packs");
            rootCommand.Subcommands.Add(validateCommand);
            rootCommand.Subcommands.Add(buildCommand);
            rootCommand.Subcommands.Add(validateTcCommand);
            rootCommand.Subcommands.Add(thunderstoreCommand);
            rootCommand.Subcommands.Add(assetsCommand);
            rootCommand.Subcommands.Add(bundlesCommand);
            rootCommand.Subcommands.Add(packCommand);
            rootCommand.Subcommands.Add(signCommand);
            rootCommand.Subcommands.Add(verifyCommand);

            ParseResult parseResultObj = rootCommand.Parse(args);

            return await parseResultObj.InvokeAsync().ConfigureAwait(false);
        }

        /// <summary>
        /// [#622] Locates schemas/pack-manifest.schema.json by walking up from the pack path,
        /// then falling back to AppContext.BaseDirectory ancestors. Throws if not found so
        /// the validate command fails loudly instead of silently skipping schema validation.
        /// </summary>
        private static string LocatePackManifestSchema(string packPath)
        {
            const string Relative = "schemas/pack-manifest.schema.json";
            var searchRoots = new List<string>();
            try { searchRoots.Add(Path.GetFullPath(packPath)); } catch { /* ignore */ }
            try { searchRoots.Add(Path.GetFullPath(Environment.CurrentDirectory)); } catch { /* ignore */ }
            try { searchRoots.Add(Path.GetFullPath(AppContext.BaseDirectory)); } catch { /* ignore */ }

            var visited = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            foreach (string root in searchRoots)
            {
                DirectoryInfo? dir = new DirectoryInfo(root);
                while (dir != null)
                {
                    if (!visited.Add(dir.FullName))
                        break;
                    string candidate = Path.Combine(dir.FullName, Relative.Replace('/', Path.DirectorySeparatorChar));
                    if (File.Exists(candidate))
                        return candidate;
                    dir = dir.Parent;
                }
            }

            throw new FileNotFoundException(
                $"[#622] Could not locate '{Relative}' walking up from '{packPath}'. " +
                "Schema validation requires the repo-tracked schemas/ directory; refusing to skip silently.");
        }

        [RequiresUnreferencedCode("Calls System.Text.Json.JsonSerializer.Serialize<TValue>(TValue, JsonSerializerOptions)")]
        private static void ValidatePack(string packPath, string format = "text")
        {
            bool jsonMode = string.Equals(format, "json", StringComparison.OrdinalIgnoreCase);
            try
            {
                if (!jsonMode)
                {
                    AnsiConsole.MarkupLine("[bold blue]PackCompiler Validate[/]");
                    AnsiConsole.MarkupLine($"Pack Path: {packPath}");
                    AnsiConsole.WriteLine();
                }

                if (!Directory.Exists(packPath))
                {
                    WriteValidationError(jsonMode, null, "Pack directory not found");
                    Environment.Exit(1);
                }

                string manifestPath = Path.Combine(packPath, "pack.yaml");
                if (!File.Exists(manifestPath))
                {
                    List<string> aggregatePackDirs = Directory
                        .GetDirectories(packPath)
                        .Where(dir => File.Exists(Path.Combine(dir, "pack.yaml")))
                        .OrderBy(dir => dir, StringComparer.OrdinalIgnoreCase)
                        .ToList();

                    if (aggregatePackDirs.Count == 0)
                    {
                        WriteValidationError(jsonMode, null, "pack.yaml not found in directory");
                        Environment.Exit(1);
                    }

                    bool aggregateSucceeded = true;
                    foreach (string childPackDir in aggregatePackDirs)
                    {
                        aggregateSucceeded &= ValidateSinglePack(childPackDir, format);
                    }

                    if (!aggregateSucceeded)
                        Environment.Exit(1);

                    return;
                }

                ValidateSinglePack(packPath, format);
            }
            catch (Exception ex)
            {
                WriteValidationError(jsonMode, null, ex.Message);
                Environment.Exit(1);
            }
        }

        [RequiresUnreferencedCode("Calls System.Text.Json.JsonSerializer.Serialize<TValue>(TValue, JsonSerializerOptions)")]
        private static bool ValidateSinglePack(string packPath, string format = "text")
        {
            bool jsonMode = string.Equals(format, "json", StringComparison.OrdinalIgnoreCase);
            try
            {
                if (!jsonMode)
                {
                    AnsiConsole.MarkupLine("[bold blue]PackCompiler Validate[/]");
                    AnsiConsole.MarkupLine($"Pack Path: {packPath}");
                    AnsiConsole.WriteLine();
                }

                string manifestPath = Path.Combine(packPath, "pack.yaml");
                if (!File.Exists(manifestPath))
                {
                    WriteValidationError(jsonMode, null, "pack.yaml not found in directory");
                    return false;
                }

                if (!jsonMode) AnsiConsole.MarkupLine("[yellow]Loading manifest...[/]");
                var loader = new PackLoader();
                var manifest = loader.LoadFromFile(manifestPath);

                string schemaPath = LocatePackManifestSchema(packPath);
                string schemaYaml = File.ReadAllText(schemaPath, Encoding.UTF8);
                string manifestYaml = File.ReadAllText(manifestPath, Encoding.UTF8);
                var schemaValidator = new NJsonSchemaValidator(new Dictionary<string, string>(StringComparer.Ordinal)
                {
                    ["pack-manifest"] = schemaYaml,
                });

                DINOForge.SDK.Validation.ValidationResult schemaResult = schemaValidator.Validate("pack-manifest", manifestYaml);
                if (!schemaResult.IsValid)
                {
                    string[] errMessages = schemaResult.Errors
                        .Select(e => string.IsNullOrEmpty(e.Path) ? e.Message : $"{e.Path}: {e.Message}")
                        .ToArray();

                    if (jsonMode)
                    {
                        WriteJsonLine(writer =>
                        {
                            writer.WriteString("status", "error");
                            writer.WriteString("pack", manifest.Id);
                            writer.WritePropertyName("errors");
                            writer.WriteStartArray();
                            foreach (string err in errMessages)
                                writer.WriteStringValue(err);
                            writer.WriteEndArray();
                        });
                    }
                    else
                    {
                        AnsiConsole.MarkupLine("[bold red]Schema validation failed:[/]");
                        foreach (string msg in errMessages)
                        {
                            AnsiConsole.MarkupLine($"  [red]- {Markup.Escape(msg)}[/]");
                        }
                    }
                    return false;
                }

                if (jsonMode)
                {
                    WriteJsonLine(writer =>
                    {
                        writer.WriteString("status", "ok");
                        writer.WriteString("pack", manifest.Id);
                        writer.WritePropertyName("errors");
                        writer.WriteStartArray();
                        writer.WriteEndArray();
                    });
                    return true;
                }

                AnsiConsole.MarkupLine("[bold]Manifest Fields:[/]");
                var table = new Table();
                table.AddColumn("Field");
                table.AddColumn("Value");
                table.AddRow("ID", manifest.Id);
                table.AddRow("Name", manifest.Name);
                table.AddRow("Version", manifest.Version);
                table.AddRow("Author", manifest.Author ?? "[dim]<not set>[/]");
                table.AddRow("Type", manifest.Type);
                table.AddRow("Description", manifest.Description ?? "[dim]<not set>[/]");
                table.AddRow("Framework Version", manifest.FrameworkVersion);
                table.AddRow("Game Version", manifest.GameVersion ?? "[dim]<not set>[/]");
                table.AddRow("Load Order", manifest.LoadOrder.ToString());

                if (manifest.DependsOn.Count > 0)
                    table.AddRow("Depends On", string.Join(", ", manifest.DependsOn));

                if (manifest.ConflictsWith.Count > 0)
                    table.AddRow("Conflicts With", string.Join(", ", manifest.ConflictsWith));

                AnsiConsole.Write(table);
                AnsiConsole.WriteLine();

                AnsiConsole.MarkupLine("[bold]Content Files:[/]");
                var contentTable = new Table();
                contentTable.AddColumn("Type");
                contentTable.AddColumn("Count");

                var contentDirs = new[] { "factions", "units", "buildings", "weapons", "doctrines", "audio", "visuals", "localization", "wave_templates", "tech_nodes", "scenarios" };
                var foundContent = false;

                foreach (var dir in contentDirs)
                {
                    string dirPath = Path.Combine(packPath, dir);
                    if (Directory.Exists(dirPath))
                    {
                        var files = Directory.GetFiles(dirPath);
                        if (files.Length > 0)
                        {
                            contentTable.AddRow(dir, files.Length.ToString());
                            foundContent = true;
                        }
                    }
                }

                if (foundContent)
                {
                    AnsiConsole.Write(contentTable);
                }
                else
                {
                    AnsiConsole.MarkupLine("[dim]No content files found[/]");
                }

                AnsiConsole.WriteLine();
                AnsiConsole.MarkupLine("[bold green]Validation successful![/]");
                return true;
            }
            catch (Exception ex)
            {
                WriteValidationError(jsonMode, null, ex.Message);
                return false;
            }
        }

        private static void WriteValidationError(bool jsonMode, string? pack, string message)
        {
            if (jsonMode)
            {
                WriteJsonLine(writer =>
                {
                    writer.WriteString("status", "error");
                    if (pack is null)
                        writer.WriteNull("pack");
                    else
                        writer.WriteString("pack", pack);
                    writer.WritePropertyName("errors");
                    writer.WriteStartArray();
                    writer.WriteStringValue(message);
                    writer.WriteEndArray();
                });
            }
            else
            {
                AnsiConsole.MarkupLine($"[bold red]Validation failed:[/] {Markup.Escape(message)}");
            }
        }

        private static void WriteJsonLine(Action<Utf8JsonWriter> write)
        {
            using var stream = new MemoryStream();
            using var writer = new Utf8JsonWriter(stream);
            writer.WriteStartObject();
            write(writer);
            writer.WriteEndObject();
            writer.Flush();
            Console.WriteLine(Encoding.UTF8.GetString(stream.ToArray()));
        }

        [RequiresUnreferencedCode("Calls System.Text.Json.JsonSerializer.Serialize<TValue>(TValue, JsonSerializerOptions)")]
        private static void BuildPack(string packPath, string? outputDir, string format = "text")
        {
            bool jsonMode = string.Equals(format, "json", StringComparison.OrdinalIgnoreCase);
            try
            {
                if (!jsonMode)
                {
                    AnsiConsole.MarkupLine("[bold blue]PackCompiler Build[/]");
                    AnsiConsole.MarkupLine($"Pack Path: {packPath}");
                    if (!string.IsNullOrEmpty(outputDir))
                        AnsiConsole.MarkupLine($"Output Directory: {outputDir}");
                    AnsiConsole.WriteLine();
                }

                if (!Directory.Exists(packPath))
                {
                    if (jsonMode)
                        WriteJsonLine(writer =>
                        {
                            writer.WriteString("status", "error");
                            writer.WritePropertyName("errors");
                            writer.WriteStartArray();
                            writer.WriteStringValue("Pack directory not found");
                            writer.WriteEndArray();
                        });
                    else
                        AnsiConsole.MarkupLine("[bold red]Error:[/] Pack directory not found");
                    Environment.Exit(1);
                }

                string manifestPath = Path.Combine(packPath, "pack.yaml");
                if (!File.Exists(manifestPath))
                {
                    if (jsonMode)
                        WriteJsonLine(writer =>
                        {
                            writer.WriteString("status", "error");
                            writer.WritePropertyName("errors");
                            writer.WriteStartArray();
                            writer.WriteStringValue("pack.yaml not found in directory");
                            writer.WriteEndArray();
                        });
                    else
                        AnsiConsole.MarkupLine("[bold red]Error:[/] pack.yaml not found in directory");
                    Environment.Exit(1);
                }

                if (!jsonMode) AnsiConsole.MarkupLine("[yellow]Validating manifest...[/]");
                var loader = new PackLoader();
                var manifest = loader.LoadFromFile(manifestPath);
                if (!jsonMode)
                {
                    AnsiConsole.MarkupLine($"[green]v[/] Manifest valid: {manifest.Name} v{manifest.Version}");
                    AnsiConsole.WriteLine();
                }

                string finalOutputDir = outputDir ?? Path.Combine(Directory.GetCurrentDirectory(), $"{manifest.Id}-{manifest.Version}");

                if (Directory.Exists(finalOutputDir))
                {
                    if (!jsonMode) AnsiConsole.MarkupLine($"[yellow]Clearing existing output directory...[/]");
                    Directory.Delete(finalOutputDir, true);
                }

                if (!jsonMode) AnsiConsole.MarkupLine($"[yellow]Copying pack to output directory...[/]");
                CopyDirectory(packPath, finalOutputDir);

                if (!jsonMode) AnsiConsole.MarkupLine("[yellow]Generating Thunderstore manifest...[/]");
                GenerateThunderstoreManifest(packPath, GetDefaultAuthor(), finalOutputDir);

                // Compute output size
                long outputSize = Directory.GetFiles(finalOutputDir, "*", SearchOption.AllDirectories)
                    .Sum(f => new FileInfo(f).Length);

                if (jsonMode)
                {
                    WriteJsonLine(writer =>
                    {
                        writer.WriteString("status", "ok");
                        writer.WriteString("output", finalOutputDir);
                        writer.WriteNumber("size", outputSize);
                    });
                }
                else
                {
                    AnsiConsole.WriteLine();
                    AnsiConsole.MarkupLine("[bold green]Build successful![/]");
                    AnsiConsole.MarkupLine($"Output: {finalOutputDir}");
                }
            }
            catch (Exception ex)
            {
                if (jsonMode)
                    WriteJsonLine(writer =>
                    {
                        writer.WriteString("status", "error");
                        writer.WritePropertyName("errors");
                        writer.WriteStartArray();
                        writer.WriteStringValue(ex.Message);
                        writer.WriteEndArray();
                    });
                else
                    AnsiConsole.MarkupLine($"[bold red]Build failed:[/] {ex.Message}");
                Environment.Exit(1);
            }
        }

        [RequiresUnreferencedCode("Calls System.Text.Json.JsonSerializer.Serialize<TValue>(TValue, JsonSerializerOptions)")]
        private static void CreateThunderstorePackage(string packPath, string author, string outputDir)
        {
            try
            {
                AnsiConsole.MarkupLine("[bold blue]Thunderstore Package Creation[/]");
                AnsiConsole.MarkupLine($"Pack Path: {packPath}");
                AnsiConsole.MarkupLine($"Author: {author}");
                AnsiConsole.MarkupLine($"Output Directory: {outputDir}");
                AnsiConsole.WriteLine();

                if (!Directory.Exists(packPath))
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] Pack directory not found");
                    Environment.Exit(1);
                    return;
                }

                string manifestPath = Path.Combine(packPath, "pack.yaml");
                if (!File.Exists(manifestPath))
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] pack.yaml not found in directory");
                    Environment.Exit(1);
                    return;
                }

                AnsiConsole.MarkupLine("[yellow]Loading manifest...[/]");
                var loader = new PackLoader();
                var manifest = loader.LoadFromFile(manifestPath);

                // Build Thunderstore package name: "Author-PackId" (sanitized to alphanumeric + dash)
                string safeName = System.Text.RegularExpressions.Regex.Replace(
                    manifest.Id, @"[^a-zA-Z0-9_]", "-");
                string tsName = $"{author}-{safeName}";

                // Map DINOForge depends_on to Thunderstore format
                var dependencies = new List<string> { "BepInEx-BepInExPack-5.4.2100" };
                if (manifest.DependsOn != null && manifest.DependsOn.Count > 0)
                {
                    foreach (string dep in manifest.DependsOn)
                    {
                        string safeDep = System.Text.RegularExpressions.Regex.Replace(dep, @"[^a-zA-Z0-9_]", "-");
                        dependencies.Add($"{author}-{safeDep}-1.0.0");
                    }
                }

                // Truncate description to 250 chars (Thunderstore limit)
                string description = manifest.Description ?? $"DINOForge pack: {manifest.Name}";
                if (description.Length > 250)
                    description = description[..247] + "...";

                // Create staging directory
                string stagingDir = Path.Combine(outputDir, tsName);
                if (Directory.Exists(stagingDir))
                {
                    AnsiConsole.MarkupLine("[yellow]Clearing existing staging directory...[/]");
                    Directory.Delete(stagingDir, true);
                }
                Directory.CreateDirectory(stagingDir);

                AnsiConsole.MarkupLine("[yellow]Copying pack contents (excluding raw assets)...[/]");

                // Copy pack files, excluding raw asset working directories
                var excludedPatterns = new[] { "assets/raw", "assets/working", @"\.blend$", @"\.fbx$", @"\.psd$" };
                CopyDirectoryExcluding(packPath, stagingDir, excludedPatterns);

                // Generate manifest.json manually (trimming-safe approach)
                AnsiConsole.MarkupLine("[yellow]Generating manifest.json...[/]");
                string manifestJsonPath = Path.Combine(stagingDir, "manifest.json");
                var manifestJson = BuildThunderstoreManifestJson(tsName, manifest.Version, GetDefaultWebsiteUrl(), description, dependencies);
                File.WriteAllText(manifestJsonPath, manifestJson, Encoding.UTF8);
                AnsiConsole.MarkupLine($"[green]✓[/] manifest.json created");

                // Check for icon.png (required by Thunderstore)
                string iconPath = Path.Combine(stagingDir, "icon.png");
                if (!File.Exists(iconPath))
                {
                    AnsiConsole.MarkupLine("[yellow]⚠[/] Warning: icon.png not found. Thunderstore requires a 256x256 PNG icon.");
                    AnsiConsole.MarkupLine($"    Place icon.png at: {Markup.Escape(iconPath)}");
                }
                else
                {
                    AnsiConsole.MarkupLine("[green]✓[/] icon.png found");
                }

                // Check for README.md (optional but recommended)
                string readmePath = Path.Combine(stagingDir, "README.md");
                if (!File.Exists(readmePath))
                {
                    AnsiConsole.MarkupLine("[yellow]ℹ[/] Info: README.md not found. Creating a basic one...");
                    string basicReadme = $@"# {manifest.Name}

{manifest.Description ?? $"A DINOForge mod pack for Diplomacy is Not an Option"}

## Installation

1. Install [BepInEx](https://valheim.thunderstore.io/package/denikson/BepInExPack/)
2. Extract this mod to your BepInEx plugins folder
3. Launch the game

## Dependencies

- BepInEx 5.4.2100+
{(manifest.DependsOn?.Count > 0 ? $"- {string.Join("\n- ", manifest.DependsOn)}" : "")}

## Version

{manifest.Version}

## Author

{author}
";
                    File.WriteAllText(readmePath, basicReadme, Encoding.UTF8);
                    AnsiConsole.MarkupLine("[green]✓[/] Basic README.md created");
                }
                else
                {
                    AnsiConsole.MarkupLine("[green]✓[/] README.md found");
                }

                // Create ZIP archive
                string zipFileName = $"{tsName}-{manifest.Version}.zip";
                string zipPath = Path.Combine(outputDir, zipFileName);

                if (File.Exists(zipPath))
                {
                    AnsiConsole.MarkupLine($"[yellow]Removing existing ZIP: {zipPath}[/]");
                    File.Delete(zipPath);
                }

                AnsiConsole.MarkupLine("[yellow]Creating ZIP archive...[/]");
                using (var zipFile = ZipFile.Open(zipPath, ZipArchiveMode.Create))
                {
                    foreach (string file in Directory.GetFiles(stagingDir, "*", SearchOption.AllDirectories))
                    {
                        string relativePath = Path.GetRelativePath(stagingDir, file);
                        // Normalize path separators for ZIP (always forward slash)
                        string zipPath_normalized = relativePath.Replace(Path.DirectorySeparatorChar, '/');
                        ZipFileExtensions.CreateEntryFromFile(zipFile, file, zipPath_normalized);
                    }
                }

                // Compute ZIP size
                long zipSize = new FileInfo(zipPath).Length;

                // Clean up staging directory
                AnsiConsole.MarkupLine("[yellow]Cleaning up staging directory...[/]");
                Directory.Delete(stagingDir, true);

                AnsiConsole.WriteLine();
                AnsiConsole.MarkupLine("[bold green]Package created successfully![/]");
                AnsiConsole.MarkupLine($"[bold]ZIP:[/] {zipPath}");
                AnsiConsole.MarkupLine($"[bold]Size:[/] {FormatBytes(zipSize)}");
                AnsiConsole.MarkupLine($"[bold]Package Name:[/] {tsName}");
                AnsiConsole.MarkupLine($"[bold]Version:[/] {manifest.Version}");
                AnsiConsole.MarkupLine($"[bold]Dependencies:[/] {dependencies.Count}");
                AnsiConsole.WriteLine();
                AnsiConsole.MarkupLine("[green]Ready to upload to Thunderstore![/]");
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[bold red]Package creation failed:[/] {ex.Message}");
                Environment.Exit(1);
            }
        }

        [RequiresUnreferencedCode("Calls System.Text.Json.JsonSerializer.Serialize<TValue>(TValue, JsonSerializerOptions)")]
        private static void GenerateThunderstoreManifest(string packPath, string author, string outputDir)
        {
            try
            {
                string manifestPath = Path.Combine(packPath, "pack.yaml");
                if (!File.Exists(manifestPath))
                {
                    AnsiConsole.MarkupLine($"[bold red]ERROR:[/] No pack.yaml found in {packPath}");
                    Environment.Exit(1);
                    return;
                }

                var loader = new PackLoader();
                var manifest = loader.LoadFromFile(manifestPath);

                // Build Thunderstore package name: "Author-PackId" (sanitized to alphanumeric + dash)
                string safeName = System.Text.RegularExpressions.Regex.Replace(
                    manifest.Id, @"[^a-zA-Z0-9_]", "-");
                string tsName = $"{author}-{safeName}";

                // Map DINOForge depends_on to Thunderstore format
                // Always include BepInEx as base dependency
                var dependencies = new List<string> { "BepInEx-BepInExPack-5.4.2100" };
                if (manifest.DependsOn != null && manifest.DependsOn.Count > 0)
                {
                    foreach (string dep in manifest.DependsOn)
                    {
                        // Convert DINOForge dep ID to Thunderstore format
                        string safeDep = System.Text.RegularExpressions.Regex.Replace(dep, @"[^a-zA-Z0-9_]", "-");
                        dependencies.Add($"{author}-{safeDep}-1.0.0");
                    }
                }

                // Truncate description to 250 chars (Thunderstore limit)
                string description = manifest.Description ?? $"DINOForge pack: {manifest.Name}";
                if (description.Length > 250)
                    description = description[..247] + "...";

                var tsManifest = new
                {
                    name = tsName,
                    version_number = manifest.Version,
                    website_url = GetDefaultWebsiteUrl(),
                    description = description,
                    dependencies = dependencies
                };

                string finalOutputDir = string.IsNullOrEmpty(outputDir) ? packPath : outputDir;
                Directory.CreateDirectory(finalOutputDir);
                string outPath = Path.Combine(finalOutputDir, "thunderstore.manifest.json");

                string json = JsonSerializer.Serialize(tsManifest, PackCompilerJsonOptions.GoFfi);
                File.WriteAllText(outPath, json, Encoding.UTF8);

                AnsiConsole.MarkupLine($"[green]✓[/] Thunderstore manifest written to: [bold]{outPath}[/]");
                AnsiConsole.MarkupLine($"  Package: [bold]{tsManifest.name}[/] v{tsManifest.version_number}");
                AnsiConsole.MarkupLine($"  Dependencies: [dim]{dependencies.Count}[/]");
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[bold red]Thunderstore generation failed:[/] {ex.Message}");
                Environment.Exit(1);
            }
        }

        private static void AssetsListBundles(string gameDir)
        {
            try
            {
                AnsiConsole.MarkupLine("[bold blue]Asset Bundle List[/]");
                AnsiConsole.MarkupLine($"Game Directory: {gameDir}");
                AnsiConsole.WriteLine();

                // NOTE: AssetService is in Runtime (BepInEx plugin context), not available in CLI
                AnsiConsole.MarkupLine("[yellow]Asset bundle inspection is not available in PackCompiler CLI.[/]");
                AnsiConsole.MarkupLine("[yellow]Use the MCP bridge tools instead: game_screenshot, game_analyze_screen[/]");
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[bold red]Error:[/] {Markup.Escape(ex.Message)}");
                Environment.Exit(1);
            }
        }

        private static void AssetsInspect(string bundlePath)
        {
            try
            {
                AnsiConsole.MarkupLine("[bold blue]Asset Bundle Inspection[/]");
                AnsiConsole.MarkupLine($"Bundle: {Markup.Escape(bundlePath)}");
                AnsiConsole.WriteLine();

                // NOTE: AssetService is in Runtime (BepInEx plugin context), not available in CLI
                AnsiConsole.MarkupLine("[yellow]Asset bundle inspection is not available in PackCompiler CLI.[/]");
                AnsiConsole.MarkupLine("[yellow]Use the MCP bridge tools instead: game_screenshot, game_analyze_screen[/]");
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[bold red]Error:[/] {Markup.Escape(ex.Message)}");
                Environment.Exit(1);
            }
        }

        private static void AssetsValidate(string modBundlePath)
        {
            try
            {
                AnsiConsole.MarkupLine("[bold blue]Mod Bundle Validation[/]");
                AnsiConsole.MarkupLine($"Bundle: {Markup.Escape(modBundlePath)}");
                AnsiConsole.WriteLine();

                // NOTE: AssetService is in Runtime (BepInEx plugin context), not available in CLI
                AnsiConsole.MarkupLine("[yellow]Asset bundle validation is not available in PackCompiler CLI.[/]");
                AnsiConsole.MarkupLine("[yellow]Use the MCP bridge tools instead: game_screenshot, game_analyze_screen[/]");
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[bold red]Error:[/] {Markup.Escape(ex.Message)}");
                Environment.Exit(1);
            }
        }

        private static string FormatBytes(long bytes)
        {
            string[] suffixes = { "B", "KB", "MB", "GB" };
            int order = 0;
            double size = bytes;
            while (size >= 1024 && order < suffixes.Length - 1)
            {
                order++;
                size /= 1024;
            }
            return $"{size:0.##} {suffixes[order]}";
        }

        private static void CopyDirectory(string sourceDir, string destDir)
        {
            Directory.CreateDirectory(destDir);

            foreach (var file in Directory.GetFiles(sourceDir))
            {
                string destFile = Path.Combine(destDir, Path.GetFileName(file));
                File.Copy(file, destFile, true);
            }

            foreach (var dir in Directory.GetDirectories(sourceDir))
            {
                string destSubDir = Path.Combine(destDir, Path.GetFileName(dir));
                CopyDirectory(dir, destSubDir);
            }
        }

        /// <summary>
        /// Copies a directory tree, excluding paths matching any of the exclusion patterns.
        /// Patterns can be directory names (e.g., "assets/raw") or file extensions (e.g., "\.blend$").
        /// </summary>
        private static void CopyDirectoryExcluding(string sourceDir, string destDir, string[] excludePatterns)
        {
            Directory.CreateDirectory(destDir);

            foreach (var file in Directory.GetFiles(sourceDir))
            {
                string relativePath = Path.GetRelativePath(sourceDir, file);
                if (ShouldExclude(relativePath, excludePatterns))
                    continue;

                string destFile = Path.Combine(destDir, Path.GetFileName(file));
                File.Copy(file, destFile, true);
            }

            foreach (var dir in Directory.GetDirectories(sourceDir))
            {
                string dirName = Path.GetFileName(dir);
                string relativePath = Path.GetRelativePath(sourceDir, dir);

                if (ShouldExclude(relativePath, excludePatterns))
                    continue;

                string destSubDir = Path.Combine(destDir, dirName);
                CopyDirectoryExcluding(dir, destSubDir, excludePatterns);
            }
        }

        /// <summary>
        /// Returns true if the given path matches any of the exclusion patterns.
        /// </summary>
        private static bool ShouldExclude(string path, string[] patterns)
        {
            // Normalize path separators for consistent matching
            string normalizedPath = path.Replace(Path.DirectorySeparatorChar, '/');

            foreach (var pattern in patterns)
            {
                // Direct substring match (e.g., "assets/raw")
                if (normalizedPath.Contains(pattern, StringComparison.OrdinalIgnoreCase))
                    return true;

                // Regex pattern match (e.g., "\.blend$")
                if (pattern.StartsWith(@"\.") || pattern.Contains("$"))
                {
                    try
                    {
                        if (System.Text.RegularExpressions.Regex.IsMatch(normalizedPath, pattern, System.Text.RegularExpressions.RegexOptions.IgnoreCase))
                            return true;
                    }
                    catch { /* skip invalid regex patterns */ }
                }
            }

            return false;
        }

        private static void ValidateTotalConversion(string manifestPath)
        {
            try
            {
                AnsiConsole.MarkupLine("[bold blue]Total Conversion Validator[/]");

                if (!File.Exists(manifestPath))
                {
                    AnsiConsole.MarkupLine($"[red]File not found:[/] {manifestPath}");
                    Environment.Exit(1);
                    return;
                }

                var deserializer = YamlLoader.Deserializer;

                string yaml = SafeFileIO.ReadText(manifestPath);
                var manifest = deserializer.Deserialize<TotalConversionManifest>(yaml);

                if (manifest == null)
                {
                    AnsiConsole.MarkupLine("[red]Failed to parse total conversion manifest[/]");
                    Environment.Exit(1);
                    return;
                }

                var result = TotalConversionValidator.Validate(manifest);
                var unreplaced = TotalConversionValidator.GetUnreplacedFactions(manifest);

                AnsiConsole.MarkupLine($"\n[bold]Pack:[/] {manifest.Name} v{manifest.Version}");
                AnsiConsole.MarkupLine($"[bold]Author:[/] {manifest.Author}");
                AnsiConsole.MarkupLine($"[bold]Theme:[/] {manifest.Theme ?? "[dim]<none>[/]"}");
                AnsiConsole.MarkupLine($"[bold]Singleton:[/] {(manifest.Singleton ? "Yes" : "No")}");

                AnsiConsole.MarkupLine($"\n[bold]Content:[/]");
                AnsiConsole.MarkupLine($"  Factions: {manifest.Factions.Count}");
                AnsiConsole.MarkupLine($"  Asset Replacements (total): {manifest.AssetReplacements.Textures.Count + manifest.AssetReplacements.Audio.Count + manifest.AssetReplacements.Ui.Count}");
                AnsiConsole.MarkupLine($"    - Textures: {manifest.AssetReplacements.Textures.Count}");
                AnsiConsole.MarkupLine($"    - Audio: {manifest.AssetReplacements.Audio.Count}");
                AnsiConsole.MarkupLine($"    - UI: {manifest.AssetReplacements.Ui.Count}");

                AnsiConsole.MarkupLine($"\n[bold]Vanilla Replacements:[/]");
                foreach (var kvp in manifest.ReplacesVanilla)
                {
                    var faction = manifest.Factions.FirstOrDefault(f => f.Id == kvp.Value);
                    string factionName = faction?.Name ?? "[red]<not found>[/]";
                    AnsiConsole.MarkupLine($"  {kvp.Key} → {kvp.Value} ({factionName})");
                }

                if (result.IsValid && unreplaced.Count == 0)
                {
                    AnsiConsole.MarkupLine($"\n[green]✓ Total conversion '{manifest.Name}' is [bold]valid[/][/]");
                }
                else
                {
                    if (result.Errors.Count > 0)
                    {
                        AnsiConsole.MarkupLine("\n[red]Errors:[/]");
                        foreach (string e in result.Errors)
                            AnsiConsole.MarkupLine($"  [red]✗[/] {Markup.Escape(e)}");
                    }

                    if (unreplaced.Count > 0)
                        AnsiConsole.MarkupLine($"\n[yellow]Unreplaced vanilla factions:[/] {string.Join(", ", unreplaced)}");

                    if (result.Errors.Count > 0)
                    {
                        AnsiConsole.MarkupLine("");
                        Environment.Exit(1);
                    }
                }

                if (result.Warnings.Count > 0)
                {
                    AnsiConsole.MarkupLine("\n[yellow]Warnings:[/]");
                    foreach (string w in result.Warnings)
                        AnsiConsole.MarkupLine($"  [yellow]⚠[/] {Markup.Escape(w)}");
                }
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[bold red]Error:[/] {Markup.Escape(ex.Message)}");
                Environment.Exit(1);
            }
        }

        private static void PackAdd(string repoUrl, string path)
        {
            // Extract repo name from URL
            string repoName = repoUrl.TrimEnd('/').Split('/').Last();
            if (repoName.EndsWith(".git")) repoName = repoName[..^4];

            string submodulePath = string.IsNullOrEmpty(path) ? $"packs/{repoName}" : path;

            AnsiConsole.MarkupLine($"[cyan]Adding pack submodule:[/] {repoUrl} → {submodulePath}");

            int result = RunGit($"submodule add {repoUrl} {submodulePath}");
            if (result != 0)
            {
                AnsiConsole.MarkupLine($"[red]Failed to add submodule. Is this a git repo?[/]");
                Environment.Exit(1);
            }

            AnsiConsole.MarkupLine($"[green]✓ Pack added:[/] {submodulePath}");
            AnsiConsole.MarkupLine("[dim]Run 'pack lock' to update packs.lock[/]");
        }

        private static void PackList()
        {
            // Read .gitmodules
            string gitmodulesPath = ".gitmodules";
            if (!File.Exists(gitmodulesPath))
            {
                AnsiConsole.MarkupLine("[yellow]No pack submodules found (.gitmodules not present)[/]");
                return;
            }

            string content = File.ReadAllText(gitmodulesPath, Encoding.UTF8);
            var lines = content.Split('\n');

            var table = new Table();
            table.AddColumn("Path");
            table.AddColumn("URL");

            string currentPath = "", currentUrl = "";
            foreach (string line in lines)
            {
                string trimmed = line.Trim();
                if (trimmed.StartsWith("path = ")) currentPath = trimmed.Substring(7);
                else if (trimmed.StartsWith("url = "))
                {
                    currentUrl = trimmed.Substring(6);
                    if (!string.IsNullOrEmpty(currentPath))
                    {
                        bool isPack = currentPath.StartsWith("packs/");
                        if (isPack) table.AddRow(currentPath, currentUrl);
                        currentPath = "";
                        currentUrl = "";
                    }
                }
            }

            AnsiConsole.Write(table);
        }

        private static void PackUpdate()
        {
            AnsiConsole.MarkupLine("[cyan]Updating all pack submodules...[/]");
            int result = RunGit("submodule update --remote --merge packs/");
            if (result == 0)
                AnsiConsole.MarkupLine("[green]✓ All packs updated[/]");
            else
                AnsiConsole.MarkupLine("[red]Some updates failed[/]");
        }

        private static void PackLock()
        {
            // Read current submodule SHAs
            string gitmodulesPath = ".gitmodules";
            if (!File.Exists(gitmodulesPath))
            {
                AnsiConsole.MarkupLine("[yellow]No submodules found[/]");
                return;
            }

            var lockEntries = new StringBuilder(512);  // Capacity ~= 4 appends × 100 chars in loop
            lockEntries.AppendLine("# packs.lock - generated by dinoforge pack lock");
            lockEntries.AppendLine($"# Generated: {DateTime.UtcNow:yyyy-MM-dd HH:mm:ss} UTC");
            lockEntries.AppendLine();

            // Get submodule status
            var psi = new ProcessStartInfo("git", "submodule status packs/")
            {
                RedirectStandardOutput = true,
                UseShellExecute = false
            };
            using var proc = Process.Start(psi);
            string? status = proc?.StandardOutput.ReadToEnd();
            proc?.WaitForExit();

            if (!string.IsNullOrEmpty(status))
            {
                foreach (string line in status.Split('\n'))
                {
                    if (string.IsNullOrWhiteSpace(line)) continue;
                    string trimmed = line.TrimStart('+', '-', ' ');
                    string[] parts = trimmed.Split(' ', 2);
                    if (parts.Length >= 2)
                        lockEntries.AppendLine($"{parts[1].Split(' ')[0]} {parts[0]}");
                }
            }

            File.WriteAllText("packs.lock", lockEntries.ToString(), Encoding.UTF8);
            AnsiConsole.MarkupLine("[green]✓ packs.lock generated[/]");
        }

        private static int RunGit(string args)
        {
            var psi = new ProcessStartInfo("git", args)
            {
                UseShellExecute = false
            };
            using var proc = Process.Start(psi);
            proc?.WaitForExit();
            return proc?.ExitCode ?? 1;
        }

        // Asset Pipeline Commands (v0.7.0+)
        [RequiresUnreferencedCode("Calls System.Text.Json.JsonSerializer.Serialize<TValue>(TValue, JsonSerializerOptions)")]
        private static void AssetImport(string packPath)
        {
            try
            {
                AnsiConsole.MarkupLine("[bold blue]Asset Import Pipeline[/]");
                AnsiConsole.MarkupLine($"Pack: {packPath}");
                AnsiConsole.WriteLine();

                string configPath = Path.Combine(packPath, "asset_pipeline.yaml");

                if (!File.Exists(configPath))
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] asset_pipeline.yaml not found");
                    Environment.Exit(1);
                    return;
                }

                var deserializer = YamlLoader.Deserializer;

                var configYaml = File.ReadAllText(configPath, Encoding.UTF8);

                // Deserialize with timeout
                var deserializeTask = Task.Run(() =>
                {
                    return deserializer.Deserialize<AssetPipelineConfig>(configYaml);
                });

                if (!deserializeTask.Wait(TimeSpan.FromSeconds(10)))
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] YAML deserialization timeout");
                    Environment.Exit(1);
                    return;
                }

                var config = deserializeTask.Result;

                if (config == null)
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] Failed to parse asset_pipeline.yaml");
                    Environment.Exit(1);
                    return;
                }

                AnsiConsole.MarkupLine($"[green]✓[/] Loaded config: Pack {config.PackId} v{config.Version}");
                AnsiConsole.MarkupLine($"  Phases: {config.Phases.Count}");

                var importService = new AssetImportService();
                int successCount = 0, failCount = 0;

                var importedDir = Path.Combine(packPath, config.AssetSettings.BasePath, "imported");
                Directory.CreateDirectory(importedDir);

                foreach (var (phaseName, phase) in config.Phases)
                {
                    AnsiConsole.MarkupLine($"\n[cyan]Phase:[/] {phaseName}");

                    foreach (var asset in phase.Models)
                    {
                        var assetPath = Path.Combine(packPath, config.AssetSettings.BasePath, asset.File);

                        try
                        {
                            if (!File.Exists(assetPath))
                            {
                                AnsiConsole.MarkupLine($"  [red]✗[/] {asset.Id}: File not found ({asset.File})");
                                failCount++;
                                continue;
                            }

                            var imported = importService.ImportAsync(asset.Id, assetPath).GetAwaiter().GetResult();

                            // Save imported asset as JSON
                            var outputPath = Path.Combine(importedDir, $"{asset.Id}.json");
                            var json = System.Text.Json.JsonSerializer.Serialize(imported, PackCompilerJsonOptions.Indented);
                            File.WriteAllText(outputPath, json, Encoding.UTF8);

                            AnsiConsole.MarkupLine($"  [green]✓[/] {asset.Id} → {outputPath}");
                            successCount++;
                        }
                        catch (Exception ex)
                        {
                            AnsiConsole.MarkupLine($"  [red]✗[/] {asset.Id}: {ex.Message}");
                            failCount++;
                        }
                    }
                }

                AnsiConsole.WriteLine();
                AnsiConsole.MarkupLine($"[bold]Results:[/] {successCount} imported, {failCount} failed");

                if (failCount > 0)
                    Environment.Exit(1);
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[bold red]Error:[/] {ex.Message}");
                Environment.Exit(1);
            }
        }

        private static void AssetValidate(string packPath)
        {
            try
            {
                AnsiConsole.MarkupLine("[bold blue]Asset Validation[/]");
                AnsiConsole.MarkupLine($"Pack: {packPath}");
                AnsiConsole.WriteLine();

                string configPath = Path.Combine(packPath, "asset_pipeline.yaml");
                if (!File.Exists(configPath))
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] asset_pipeline.yaml not found");
                    Environment.Exit(1);
                    return;
                }

                var deserializer = YamlLoader.Deserializer;

                var configYaml = File.ReadAllText(configPath, Encoding.UTF8);

                AnsiConsole.MarkupLine("[dim]Parsing configuration...[/]");
                AssetPipelineConfig? config = null;
                try
                {
                    config = deserializer.Deserialize<AssetPipelineConfig>(configYaml);
                }
                catch (Exception ex)
                {
                    AnsiConsole.MarkupLine($"[bold red]Error:[/] Failed to parse YAML: {ex.Message}");
                    Environment.Exit(1);
                    return;
                }

                if (config == null)
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] Failed to parse asset_pipeline.yaml");
                    Environment.Exit(1);
                    return;
                }

                var validationService = new AssetValidationService();
                var configResult = validationService.ValidateConfiguration(config);

                if (!configResult.IsValid)
                {
                    AnsiConsole.MarkupLine("[bold red]Configuration invalid:[/]");
                    foreach (var error in configResult.Errors)
                        AnsiConsole.MarkupLine($"  [red]✗[/] {error}");
                    Environment.Exit(1);
                    return;
                }

                AnsiConsole.MarkupLine("[green]✓[/] Configuration valid");
                AnsiConsole.WriteLine();

                if (configResult.Warnings.Count > 0)
                {
                    AnsiConsole.MarkupLine("[yellow]Warnings:[/]");
                    foreach (var warning in configResult.Warnings)
                        AnsiConsole.MarkupLine($"  [yellow]⚠[/] {warning}");
                }

                AnsiConsole.MarkupLine($"\n[bold green]Validation passed![/]");
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[bold red]Error:[/] {ex.Message}");
                Environment.Exit(1);
            }
        }

        [RequiresUnreferencedCode("Calls System.Text.Json.JsonSerializer.Serialize<TValue>(TValue, JsonSerializerOptions)")]
        private static void AssetOptimize(string packPath)
        {
            try
            {
                AnsiConsole.MarkupLine("[bold blue]Asset Optimization Pipeline (LOD Generation)[/]");
                AnsiConsole.MarkupLine($"Pack: {packPath}");
                AnsiConsole.WriteLine();

                string configPath = Path.Combine(packPath, "asset_pipeline.yaml");
                if (!File.Exists(configPath))
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] asset_pipeline.yaml not found");
                    Environment.Exit(1);
                    return;
                }

                var deserializer = YamlLoader.Deserializer;

                var configYaml = File.ReadAllText(configPath, Encoding.UTF8);

                // Deserialize with timeout
                var deserializeTask = Task.Run(() =>
                {
                    return deserializer.Deserialize<AssetPipelineConfig>(configYaml);
                });

                if (!deserializeTask.Wait(TimeSpan.FromSeconds(10)))
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] YAML deserialization timeout");
                    Environment.Exit(1);
                    return;
                }

                var config = deserializeTask.Result;

                if (config == null)
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] Failed to parse asset_pipeline.yaml");
                    Environment.Exit(1);
                    return;
                }

                AnsiConsole.MarkupLine($"[green]✓[/] Loaded config: Pack {config.PackId} v{config.Version}");
                AnsiConsole.MarkupLine($"  Phases: {config.Phases.Count}");
                AnsiConsole.WriteLine();

                var importService = new AssetImportService();
                var optimizationService = new AssetOptimizationService();
                int successCount = 0, failCount = 0;

                var optimizedDir = Path.Combine(packPath, config.AssetSettings.OptimizedPath);
                Directory.CreateDirectory(optimizedDir);

                foreach (var (phaseName, phase) in config.Phases)
                {
                    AnsiConsole.MarkupLine($"[cyan]Phase:[/] {phaseName}");

                    foreach (var assetDef in phase.Models)
                    {
                        var assetPath = Path.Combine(packPath, config.AssetSettings.BasePath, assetDef.File);

                        try
                        {
                            if (!File.Exists(assetPath))
                            {
                                AnsiConsole.MarkupLine($"  [red]✗[/] {assetDef.Id}: File not found");
                                failCount++;
                                continue;
                            }

                            // Import asset
                            var imported = importService.ImportAsync(assetDef.Id, assetPath).GetAwaiter().GetResult();

                            // Optimize (generate LODs)
                            var sw = Stopwatch.StartNew();
                            var optimized = optimizationService.OptimizeAsync(imported, assetDef).GetAwaiter().GetResult();
                            sw.Stop();

                            // Save optimized LOD data
                            var outputPath = Path.Combine(optimizedDir, $"{assetDef.Id}_optimized.json");
                            var json = System.Text.Json.JsonSerializer.Serialize(optimized, PackCompilerJsonOptions.Indented);
                            File.WriteAllText(outputPath, json, Encoding.UTF8);

                            AnsiConsole.MarkupLine($"  [green]✓[/] {assetDef.Id}: LOD0={optimized.LOD0.TriangleCount}, LOD1={optimized.LOD1.TriangleCount}, LOD2={optimized.LOD2.TriangleCount} ({sw.ElapsedMilliseconds}ms) → {outputPath}");
                            successCount++;
                        }
                        catch (Exception ex)
                        {
                            AnsiConsole.MarkupLine($"  [red]✗[/] {assetDef.Id}: {ex.Message}");
                            failCount++;
                        }
                    }
                }

                AnsiConsole.WriteLine();
                AnsiConsole.MarkupLine($"[bold]Results:[/] {successCount} optimized, {failCount} failed");

                if (failCount > 0)
                    Environment.Exit(1);
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[bold red]Error:[/] {ex.Message}");
                Environment.Exit(1);
            }
        }

        [RequiresUnreferencedCode("Calls System.Text.Json.JsonSerializer.Serialize<TValue>(TValue, JsonSerializerOptions)")]
        private static void AssetGenerate(string packPath)
        {
            try
            {
                AnsiConsole.MarkupLine("[bold blue]Prefab Generation[/]");
                AnsiConsole.MarkupLine($"Pack: {packPath}");
                AnsiConsole.WriteLine();

                string configPath = Path.Combine(packPath, "asset_pipeline.yaml");
                if (!File.Exists(configPath))
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] asset_pipeline.yaml not found");
                    Environment.Exit(1);
                    return;
                }

                var deserializer = YamlLoader.Deserializer;

                var configYaml = File.ReadAllText(configPath, Encoding.UTF8);

                // Deserialize with timeout
                var deserializeTask = Task.Run(() =>
                {
                    return deserializer.Deserialize<AssetPipelineConfig>(configYaml);
                });

                if (!deserializeTask.Wait(TimeSpan.FromSeconds(10)))
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] YAML deserialization timeout");
                    Environment.Exit(1);
                    return;
                }

                var config = deserializeTask.Result;

                if (config == null)
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] Failed to parse asset_pipeline.yaml");
                    Environment.Exit(1);
                    return;
                }

                AnsiConsole.MarkupLine($"[green]✓[/] Loaded config: Pack {config.PackId} v{config.Version}");
                AnsiConsole.WriteLine();

                var importService = new AssetImportService();
                var optimizationService = new AssetOptimizationService();
                var prefabService = new PrefabGenerationService();
                var addressablesService = new AddressablesService();

                string outputDir = Path.Combine(packPath, config.Build.OutputDirectory);
                Directory.CreateDirectory(outputDir);

                int successCount = 0, failCount = 0;
                var allAssets = new List<(OptimizedAsset, AssetDefinition)>();

                foreach (var (phaseName, phase) in config.Phases)
                {
                    AnsiConsole.MarkupLine($"[cyan]Phase:[/] {phaseName}");

                    foreach (var assetDef in phase.Models)
                    {
                        var assetPath = Path.Combine(packPath, config.AssetSettings.BasePath, assetDef.File);

                        try
                        {
                            if (!File.Exists(assetPath))
                            {
                                AnsiConsole.MarkupLine($"  [red]✗[/] {assetDef.Id}: File not found");
                                failCount++;
                                continue;
                            }

                            // Import and optimize
                            var imported = importService.ImportAsync(assetDef.Id, assetPath).GetAwaiter().GetResult();
                            var optimized = optimizationService.OptimizeAsync(imported, assetDef).GetAwaiter().GetResult();

                            // Generate prefab
                            string prefabPath = Path.Combine(outputDir, $"{assetDef.Id}.prefab");
                            prefabService.GeneratePrefabAsync(optimized, assetDef, prefabPath).GetAwaiter().GetResult();

                            allAssets.Add((optimized, assetDef));
                            AnsiConsole.MarkupLine($"  [green]✓[/] {assetDef.Id}: prefab generated");
                            successCount++;
                        }
                        catch (Exception ex)
                        {
                            AnsiConsole.MarkupLine($"  [red]✗[/] {assetDef.Id}: {ex.Message}");
                            failCount++;
                        }
                    }
                }

                // Generate Addressables catalog
                AnsiConsole.MarkupLine("\n[cyan]Generating Addressables catalog...[/]");
                string catalogPath = Path.Combine(outputDir, "addressables_catalog.txt");
                addressablesService.GenerateCatalogAsync(allAssets, catalogPath).GetAwaiter().GetResult();
                AnsiConsole.MarkupLine($"[green]✓[/] Catalog: {catalogPath}");

                // Generate asset groups
                AnsiConsole.MarkupLine("[cyan]Generating asset groups...[/]");
                foreach (var (optimized, assetDef) in allAssets)
                {
                    addressablesService.GenerateAssetGroupAsync(optimized, assetDef, outputDir).GetAwaiter().GetResult();
                    AnsiConsole.MarkupLine($"[green]✓[/] {assetDef.Id}_group.yaml");
                }

                AnsiConsole.WriteLine();
                AnsiConsole.MarkupLine($"[bold]Results:[/] {successCount} prefabs generated, {failCount} failed");
                AnsiConsole.MarkupLine($"[bold]Output:[/] {outputDir}");

                if (failCount > 0)
                    Environment.Exit(1);
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[bold red]Error:[/] {ex.Message}");
                Environment.Exit(1);
            }
        }

        [RequiresUnreferencedCode("Calls trim-sensitive asset pipeline helpers that use System.Text.Json serialization.")]
        private static void AssetBuild(string packPath)
        {
            try
            {
                var sw = Stopwatch.StartNew();

                AnsiConsole.MarkupLine("[bold cyan]Asset Pipeline: Full Build[/]");
                AnsiConsole.MarkupLine("v0.7.0 + v0.8.0: import → validate → optimize → generate");
                AnsiConsole.WriteLine();

                // Step 1: Import
                AnsiConsole.MarkupLine("[cyan]Step 1: Import Assets[/]");
                AssetImport(packPath);

                // Step 2: Validate
                AnsiConsole.MarkupLine("\n[cyan]Step 2: Validate Configuration[/]");
                AssetValidate(packPath);

                // Step 3: Optimize (LOD generation)
                AnsiConsole.MarkupLine("\n[cyan]Step 3: Generate LOD Variants[/]");
                AssetOptimize(packPath);

                // Step 4: Generate (prefabs + addressables)
                AnsiConsole.MarkupLine("\n[cyan]Step 4: Generate Prefabs & Addressables[/]");
                AssetGenerate(packPath);

                sw.Stop();
                AnsiConsole.WriteLine();
                AnsiConsole.MarkupLine($"[bold green]Pipeline complete![/] ({sw.Elapsed.TotalSeconds:F1}s)");
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[bold red]Build failed:[/] {ex.Message}");
                Environment.Exit(1);
            }
        }

        /// <summary>
        /// Builds Thunderstore manifest.json manually to avoid reflection-based serialization issues.
        /// Trimming-safe: doesn't rely on anonymous type reflection.
        /// </summary>
        private static string BuildThunderstoreManifestJson(string name, string version, string websiteUrl, string description, List<string> dependencies)
        {
            using var stream = new MemoryStream();
            using var writer = new Utf8JsonWriter(stream, new JsonWriterOptions { Indented = true });

            writer.WriteStartObject();
            writer.WriteString("name", name);
            writer.WriteString("version_number", version);
            writer.WriteString("website_url", websiteUrl);
            writer.WriteString("description", description);

            writer.WritePropertyName("dependencies");
            writer.WriteStartArray();
            foreach (var dep in dependencies)
            {
                writer.WriteStringValue(dep);
            }
            writer.WriteEndArray();

            writer.WriteEndObject();
            writer.Flush();

            return Encoding.UTF8.GetString(stream.ToArray());
        }

        private static string GetDefaultAuthor()
        {
            return Environment.GetEnvironmentVariable("DINOFORGE_AUTHOR") ?? "DINOForge";
        }

        private static string GetDefaultWebsiteUrl()
        {
            return Environment.GetEnvironmentVariable("DINOFORGE_WEBSITE_URL") ?? "https://github.com/DINOForge/DINOForge";
        }

        /// <summary>
        /// Signs a pack with an RSA private key and writes the signature to pack.signature.
        /// </summary>
        private static void SignPack(string packPath, string keyPath, string outputDir)
        {
            try
            {
                AnsiConsole.MarkupLine("[bold blue]PackCompiler Sign[/]");
                AnsiConsole.MarkupLine($"Pack Path: {packPath}");
                AnsiConsole.MarkupLine($"Key File: {keyPath}");
                AnsiConsole.WriteLine();

                if (!Directory.Exists(packPath))
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] Pack directory not found");
                    Environment.Exit(1);
                }

                if (!File.Exists(keyPath))
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] Private key file not found");
                    Environment.Exit(1);
                }

                // Load the private key from PEM file
                var keyContent = File.ReadAllText(keyPath, Encoding.UTF8);
                var rsa = RSA.Create();
                rsa.ImportFromPem(keyContent.ToCharArray());

                // Sign the pack
                AnsiConsole.MarkupLine("[cyan]Computing pack hash...[/]");
                var packHash = PackSigner.ComputePackHash(packPath);
                AnsiConsole.MarkupLine($"Pack hash: {packHash}");

                AnsiConsole.MarkupLine("[cyan]Signing pack...[/]");
                var signature = PackSigner.SignPack(packPath, rsa);

                // Write signature to file
                var outputPath = Path.Combine(outputDir, "pack.signature");
                Directory.CreateDirectory(outputDir);
                File.WriteAllText(outputPath, signature, Encoding.UTF8);

                AnsiConsole.MarkupLine($"[bold green]Success![/] Signature written to: {outputPath}");
                AnsiConsole.MarkupLine($"Signature: {signature[..Math.Min(64, signature.Length)]}...");
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[bold red]Error:[/] {ex.Message}");
                Environment.Exit(1);
            }
        }

        /// <summary>
        /// Verifies a pack's signature.
        /// </summary>
        private static void VerifyPack(string packPath, string? trustedKeysFile)
        {
            try
            {
                AnsiConsole.MarkupLine("[bold blue]PackCompiler Verify[/]");
                AnsiConsole.MarkupLine($"Pack Path: {packPath}");
                if (trustedKeysFile != null)
                {
                    AnsiConsole.MarkupLine($"Trusted Keys: {trustedKeysFile}");
                }
                AnsiConsole.WriteLine();

                if (!Directory.Exists(packPath))
                {
                    AnsiConsole.MarkupLine("[bold red]Error:[/] Pack directory not found");
                    Environment.Exit(1);
                }

                var verifier = new PackVerifier();

                // Load trusted keys if provided
                int trustedCount = 0;
                if (!string.IsNullOrEmpty(trustedKeysFile) && File.Exists(trustedKeysFile))
                {
                    trustedCount = verifier.LoadTrustedKeys(trustedKeysFile);
                    AnsiConsole.MarkupLine($"[cyan]Loaded {trustedCount} trusted author(s)[/]");
                }

                // Verify the pack
                AnsiConsole.MarkupLine("[cyan]Verifying pack signature...[/]");
                var result = verifier.Verify(packPath);

                AnsiConsole.WriteLine();
                switch (result.Status)
                {
                    case SignatureStatus.Unsigned:
                        AnsiConsole.MarkupLine("[yellow]⚠ Unsigned:[/] " + result.Message);
                        break;
                    case SignatureStatus.VerifiedAuthor:
                        AnsiConsole.MarkupLine("[bold green]✓ Verified:[/] " + result.Message);
                        break;
                    case SignatureStatus.UnknownAuthor:
                        AnsiConsole.MarkupLine("[yellow]⚠ Unknown Author:[/] " + result.Message);
                        break;
                    case SignatureStatus.TamperedSignatureMismatch:
                        AnsiConsole.MarkupLine("[bold red]✗ Tampered:[/] " + result.Message);
                        break;
                    case SignatureStatus.VerificationError:
                        AnsiConsole.MarkupLine("[bold red]✗ Error:[/] " + result.Message);
                        break;
                }
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[bold red]Error:[/] {ex.Message}");
                Environment.Exit(1);
            }
        }
    }
}
