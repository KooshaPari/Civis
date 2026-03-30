#nullable enable
using System.CommandLine;
using System.Text.Json;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.GameControlCli;

/// <summary>
/// DINOForge Game Control CLI - Standalone command-line interface for checking game state.
/// Communicates directly with the running game via named pipes (GameClient).
/// Does NOT interact with the screen or other windows.
/// </summary>
public static class Program
{
    /// <summary>When true, commands output machine-readable JSON instead of human-readable text.</summary>
    public static bool JsonOutput { get; set; }

    public static async Task<int> Main(string[] args)
    {
        if (args.Length == 0)
        {
            ShowHelp();
            return 0;
        }

        // Global --format json flag (accepted before the command)
        var remainingArgs = args;
        if (args.Length > 1 && args[0] == "--format" && args[1] == "json")
        {
            JsonOutput = true;
            remainingArgs = args.Skip(2).ToArray();
        }
        else if (args.Any(a => a == "--format" && a == "json"))
        {
            JsonOutput = true;
            remainingArgs = args.Where(a => !(a == "--format" || a == "json")).ToArray();
        }
        else if (args.Contains("--format=json"))
        {
            JsonOutput = true;
            remainingArgs = args.Where(a => a != "--format=json").ToArray();
        }

        if (remainingArgs.Length == 0)
        {
            ShowHelp();
            return 0;
        }

        string command = remainingArgs[0];
        return command switch
        {
            "status" => await HandleStatusCommand(),
            "ping" => await HandlePingCommand(),
            "wait-world" => await HandleWaitWorldCommand(),
            "resources" => await HandleResourcesCommand(),
            "screenshot" => await HandleScreenshotCommand(remainingArgs.Skip(1).FirstOrDefault()),
            "catalog" => await HandleCatalogCommand(remainingArgs.Skip(1).FirstOrDefault()),
            "entities" => await HandleEntitiesCommand(remainingArgs.Skip(1).FirstOrDefault()),
            "load-scene" => await HandleLoadSceneCommand(remainingArgs.Skip(1).FirstOrDefault()),
            "start-game" => await HandleStartGameCommand(),
            "list-saves" => await HandleListSavesCommand(),
            "load-save" => await HandleLoadSaveCommand(remainingArgs.Skip(1).FirstOrDefault()),
            "dismiss" => await HandleDismissCommand(),
            "click-button" => await HandleClickButtonCommand(remainingArgs.Skip(1).FirstOrDefault()),
            "toggle-ui" => await HandleToggleUiCommand(remainingArgs.Skip(1).FirstOrDefault()),
            "scan-scene" => await HandleScanSceneCommand(remainingArgs.Skip(1).FirstOrDefault()),
            "invoke-method" => await HandleInvokeMethodCommand(remainingArgs.Skip(1).ToArray()),
            "ui-tree" => await HandleUiTreeCommand(remainingArgs.Skip(1).FirstOrDefault()),
            "demo" => await HandleDemoCommand(),
            // JSON-output bridge commands (used by Python MCP server)
            "get-stat" => await HandleGetStatCommand(remainingArgs.Skip(1).ToArray()),
            "apply-override" => await HandleApplyOverrideCommand(remainingArgs.Skip(1).ToArray()),
            "get-component-map" => await HandleGetComponentMapCommand(remainingArgs.Skip(1).FirstOrDefault()),
            "reload-packs" => await HandleReloadPacksCommand(remainingArgs.Skip(1).FirstOrDefault()),
            "verify-mod" => await HandleVerifyModCommand(remainingArgs.Skip(1).FirstOrDefault()),
            "dump-state" => await HandleDumpStateJsonCommand(remainingArgs.Skip(1).FirstOrDefault()),
            "--help" or "-h" => ShowHelpAndReturn(0),
            _ => ShowHelpAndReturn(1)
        };
    }

    private static void ShowHelp()
    {
        AnsiConsole.MarkupLine("[cyan bold]DINOForge Game Control CLI[/]");
        AnsiConsole.MarkupLine("[yellow]Direct game process communication via named pipes[/]");
        AnsiConsole.MarkupLine("");
        AnsiConsole.MarkupLine("[green]Commands:[/]");
        AnsiConsole.MarkupLine("  status           - Check game connection and status");
        AnsiConsole.MarkupLine("  ping             - Ping the game bridge (no ECS queries)");
        AnsiConsole.MarkupLine("  wait-world       - Wait for ECS world to be ready");
        AnsiConsole.MarkupLine("  resources        - Show current resource values");
        AnsiConsole.MarkupLine("  screenshot       - Capture in-game screenshot");
        AnsiConsole.MarkupLine("  catalog <cat>      - Dump game catalog (units/buildings/projectiles)");
        AnsiConsole.MarkupLine("  entities <comp>    - Query entities by component type");
        AnsiConsole.MarkupLine("  load-scene <name>  - Load a game scene by name/index (11 scenes: level0-level9, etc.)");
        AnsiConsole.MarkupLine("  start-game         - Trigger game world load via ECS singleton (bypasses menu)");
        AnsiConsole.MarkupLine("  list-saves         - List save files discovered by the bridge");
        AnsiConsole.MarkupLine("  load-save <name>   - Load a save by name (default: AUTOSAVE_1)");
        AnsiConsole.MarkupLine("  dismiss            - Dismiss 'Press Any Key to Continue' loading screen");
        AnsiConsole.MarkupLine("  click-button <name>   - Click a named Unity UI button (e.g. DINOForge_ModsButton)");
        AnsiConsole.MarkupLine("  toggle-ui <target>    - Toggle DINOForge UI: modmenu (F10) or debug (F9)");
        AnsiConsole.MarkupLine("  scan-scene <filter>   - Dump active MonoBehaviours + their void() methods");
        AnsiConsole.MarkupLine("  invoke-method <target> <method> - Call a void() method on matching MB");
        AnsiConsole.MarkupLine("  ui-tree <selector>    - Snapshot the live Unity UI hierarchy (Playwright-style DOM)");
        AnsiConsole.MarkupLine("  demo             - Full end-to-end demo: menu → mods → F9/F10 → save → gameplay");
        AnsiConsole.MarkupLine("  --help, -h       - Show this help");
        AnsiConsole.MarkupLine("");
        AnsiConsole.MarkupLine("[grey]JSON-output commands (for MCP/scripting):[/]");
        AnsiConsole.MarkupLine("  get-stat <sdk_path> <idx>            - Read stat value by SDK path (JSON)");
        AnsiConsole.MarkupLine("  apply-override <sdk_path> <value> <mode> <filter> - Apply stat override (JSON)");
        AnsiConsole.MarkupLine("  get-component-map <sdk_path>         - SDK-to-ECS component mappings (JSON)");
        AnsiConsole.MarkupLine("  reload-packs <path>                  - Reload content packs from disk (JSON)");
        AnsiConsole.MarkupLine("  verify-mod <pack_path>               - End-to-end mod verification (JSON)");
        AnsiConsole.MarkupLine("  dump-state <category>                - ECS state snapshot as JSON");
    }

    private static int ShowHelpAndReturn(int code)
    {
        if (code != 0) AnsiConsole.MarkupLine("[red]Invalid command[/]");
        ShowHelp();
        return code;
    }

    private static async Task<int> HandleStatusCommand()
    {
        // Short read timeout — bridge may be restarting after scene transition abort.
        // If first attempt fails, retry with a fresh connection.
        using var client = new GameClient(new GameClientOptions { ReadTimeoutMs = 5000 });
        try
        {
            await client.ConnectAsync();
            var status = await client.StatusAsync();

            if (JsonOutput)
            {
                var json = new
                {
                    success = true,
                    Running = status.Running,
                    WorldReady = status.WorldReady,
                    WorldName = status.WorldName,
                    EntityCount = status.EntityCount,
                    ModPlatformReady = status.ModPlatformReady,
                    LoadedPacks = status.LoadedPacks,
                    Version = status.Version
                };
                Console.WriteLine(JsonSerializer.Serialize(json));
            }
            else
            {
                AnsiConsole.MarkupLine("[green]✓[/] Connected to game bridge");
                AnsiConsole.MarkupLine($"[cyan]Running:[/] {status.Running}");
                AnsiConsole.MarkupLine($"[cyan]World ready:[/] {status.WorldReady}");
                AnsiConsole.MarkupLine($"[cyan]World name:[/] {status.WorldName}");
                AnsiConsole.MarkupLine($"[cyan]Entity count:[/] {status.EntityCount}");
                AnsiConsole.MarkupLine($"[cyan]Mod platform ready:[/] {status.ModPlatformReady}");
                AnsiConsole.MarkupLine($"[cyan]Loaded packs:[/] {status.LoadedPacks.Count}");
                foreach (var pack in status.LoadedPacks)
                {
                    AnsiConsole.MarkupLine($"  - {pack}");
                }
                AnsiConsole.MarkupLine($"[cyan]Version:[/] {status.Version}");
            }

            client.Disconnect();
            return 0;
        }
        catch (Exception ex)
        {
            // If the request timed out (e.g., bridge thread abort), check the fallback response file.
            string fallbackPath = Path.Combine(
                Path.GetTempPath(), "DINOForge", "dinoforge_bridge_fallback.txt");
            if (File.Exists(fallbackPath))
            {
                try
                {
                    string fallback = File.ReadAllText(fallbackPath).Trim();
                    if (JsonOutput)
                    {
                        Console.WriteLine(JsonSerializer.Serialize(new { success = true, raw = fallback }));
                    }
                    else
                    {
                        AnsiConsole.MarkupLine("[yellow]⚠[/] Bridge response via fallback file (timed out waiting for live response):");
                        AnsiConsole.MarkupLine($"[cyan]  {fallback}[/]");
                    }
                    File.Delete(fallbackPath);
                    return 0;
                }
                catch { }
            }

            if (JsonOutput)
            {
                Console.WriteLine(JsonSerializer.Serialize(new { success = false, error = ex.Message }));
            }
            else
            {
                AnsiConsole.MarkupLine($"[red]✗ Error:[/] {ex.Message}");
            }
            return 1;
        }
    }

    private static async Task<int> HandlePingCommand()
    {
        using var client = new GameClient(new GameClientOptions { ReadTimeoutMs = 5000 });
        try
        {
            await client.ConnectAsync();
            var ping = await client.PingAsync();

            if (JsonOutput)
            {
                Console.WriteLine(JsonSerializer.Serialize(new
                {
                    success = true,
                    Running = true,
                    Pong = ping.Pong,
                    Version = ping.Version,
                    UptimeSeconds = ping.UptimeSeconds
                }));
            }
            else
            {
                AnsiConsole.MarkupLine("[green]✓[/] Connected to game bridge");
                AnsiConsole.MarkupLine($"[cyan]Pong:[/] {ping.Pong}");
                AnsiConsole.MarkupLine($"[cyan]Version:[/] {ping.Version}");
                AnsiConsole.MarkupLine($"[cyan]Uptime:[/] {ping.UptimeSeconds:F1}s");
            }

            client.Disconnect();
            return 0;
        }
        catch (Exception ex)
        {
            if (JsonOutput)
            {
                Console.WriteLine(JsonSerializer.Serialize(new { success = false, error = ex.Message }));
            }
            else
            {
                AnsiConsole.MarkupLine($"[red]✗ Error:[/] {ex.Message}");
            }
            return 1;
        }
    }

    private static async Task<int> HandleWaitWorldCommand()
    {
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();

            await AnsiConsole.Progress()
                .StartAsync(async progress =>
                {
                    var task = progress.AddTask("[cyan]Waiting for ECS world...[/]");
                    await client.WaitForWorldAsync(30000);
                    task.Increment(100);
                });

            AnsiConsole.MarkupLine("[green]✓[/] ECS World is ready");
            client.Disconnect();
            return 0;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {ex.Message}");
            return 1;
        }
    }

    private static async Task<int> HandleResourcesCommand()
    {
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            var result = await client.GetResourcesAsync();

            AnsiConsole.MarkupLine("[cyan]Game Resources:[/]");
            AnsiConsole.MarkupLine($"  Food:   [yellow]{result.Food}[/]");
            AnsiConsole.MarkupLine($"  Wood:   [yellow]{result.Wood}[/]");
            AnsiConsole.MarkupLine($"  Stone:  [yellow]{result.Stone}[/]");
            AnsiConsole.MarkupLine($"  Iron:   [yellow]{result.Iron}[/]");
            AnsiConsole.MarkupLine($"  Money:  [yellow]{result.Money}[/]");
            AnsiConsole.MarkupLine($"  Souls:  [yellow]{result.Souls}[/]");
            AnsiConsole.MarkupLine($"  Bones:  [yellow]{result.Bones}[/]");
            AnsiConsole.MarkupLine($"  Spirit: [yellow]{result.Spirit}[/]");

            client.Disconnect();
            return 0;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {ex.Message}");
            return 1;
        }
    }

    private static async Task<int> HandleScreenshotCommand(string? outputPath)
    {
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();

            DINOForge.Bridge.Protocol.ScreenshotResult ssResult = null!;
            await AnsiConsole.Progress()
                .StartAsync(async progress =>
                {
                    var task = progress.AddTask("[cyan]Taking screenshot...[/]");
                    ssResult = await client.ScreenshotAsync(outputPath);
                    task.Increment(100);
                });

            AnsiConsole.MarkupLine($"[green]✓[/] Screenshot saved: {ssResult.Path}");
            client.Disconnect();
            return 0;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {ex.Message}");
            return 1;
        }
    }

    private static async Task<int> HandleCatalogCommand(string? category)
    {
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            var result = await client.DumpStateAsync(category);

            AnsiConsole.MarkupLine($"[cyan]Catalog dump ({category ?? "all"}):[/]");

            if (result.Units.Count > 0)
            {
                AnsiConsole.MarkupLine("[cyan]Units:[/]");
                foreach (var entry in result.Units.Take(15))
                {
                    AnsiConsole.MarkupLine($"  {entry.InferredId}: {entry.EntityCount} entities");
                }
            }

            if (result.Buildings.Count > 0)
            {
                AnsiConsole.MarkupLine("[cyan]Buildings:[/]");
                foreach (var entry in result.Buildings.Take(15))
                {
                    AnsiConsole.MarkupLine($"  {entry.InferredId}: {entry.EntityCount} entities");
                }
            }

            if (result.Projectiles.Count > 0)
            {
                AnsiConsole.MarkupLine($"[cyan]Projectiles: {result.Projectiles.Count}[/]");
            }

            client.Disconnect();
            return 0;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {ex.Message}");
            return 1;
        }
    }

    private static async Task<int> HandleEntitiesCommand(string? componentType)
    {
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            var result = await client.QueryEntitiesAsync(componentType);

            AnsiConsole.MarkupLine("[green]✓[/] Query complete");

            client.Disconnect();
            return 0;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {ex.Message}");
            return 1;
        }
    }

    private static async Task<int> HandleLoadSceneCommand(string? sceneName)
    {
        sceneName ??= "Sandbox";
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            AnsiConsole.MarkupLine($"[cyan]Loading scene:[/] {sceneName}");
            var result = await client.LoadSceneAsync(sceneName);
            if (result.Success)
            {
                AnsiConsole.MarkupLine($"[green]✓[/] Scene load dispatched: {result.Scene}");
                if (result.SceneCount > 0)
                    AnsiConsole.MarkupLine($"[cyan]Total scenes in build:[/] {result.SceneCount}");
            }
            else
                AnsiConsole.MarkupLine($"[red]✗[/] Scene load failed");
            client.Disconnect();
            return result.Success ? 0 : 1;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {ex.Message}");
            return 1;
        }
    }

    private static async Task<int> HandleStartGameCommand()
    {
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            AnsiConsole.MarkupLine("[cyan]Triggering game world load via ECS singleton...[/]");
            var result = await client.StartGameAsync();
            if (result.Success)
                AnsiConsole.MarkupLine($"[green]✓[/] {result.Message}");
            else
                AnsiConsole.MarkupLine($"[red]✗[/] {result.Message}");
            client.Disconnect();
            return result.Success ? 0 : 1;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {ex.Message}");
            return 1;
        }
    }

    private static async Task<int> HandleDismissCommand()
    {
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            AnsiConsole.MarkupLine("[cyan]Dismissing loading screen...[/]");
            var result = await client.DismissLoadScreenAsync();
            string msg = Markup.Escape(result.Message ?? "");
            if (result.Success)
                AnsiConsole.MarkupLine($"[green]✓[/] {msg}");
            else
                AnsiConsole.MarkupLine($"[red]✗[/] {msg}");
            client.Disconnect();
            return result.Success ? 0 : 1;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {Markup.Escape(ex.Message)}");
            return 1;
        }
    }

    private static async Task<int> HandleListSavesCommand()
    {
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            AnsiConsole.MarkupLine("[cyan]Querying save files...[/]");
            var result = await client.ListSavesAsync();
            AnsiConsole.MarkupLine($"[cyan]Persistent data path:[/] {result["persistentDataPath"]}");
            AnsiConsole.MarkupLine($"[cyan]Save dir:[/] {result["saveDir"]} (exists: {result["saveDirExists"]})");
            AnsiConsole.MarkupLine($"[cyan]Data path:[/] {result["dataPath"]}");
            AnsiConsole.MarkupLine($"[cyan]Save manager:[/] {result["saveManagerType"]}");
            var saves = result["saves"]?.ToObject<List<string>>() ?? new List<string>();
            AnsiConsole.MarkupLine($"[green]Found {saves.Count} save(s):[/]");
            foreach (var s in saves)
                AnsiConsole.MarkupLine($"  - {s}");
            client.Disconnect();
            return 0;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {ex.Message}");
            return 1;
        }
    }

    private static async Task<int> HandleLoadSaveCommand(string? saveName)
    {
        saveName ??= "AUTOSAVE_1";
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            AnsiConsole.MarkupLine($"[cyan]Loading save '{saveName}'...[/]");
            var result = await client.LoadSaveAsync(saveName);
            string msg = Markup.Escape(result.Message ?? "");
            if (result.Success)
                AnsiConsole.MarkupLine($"[green]✓[/] {msg}");
            else
                AnsiConsole.MarkupLine($"[red]✗[/] {msg}");
            client.Disconnect();
            return result.Success ? 0 : 1;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {ex.Message}");
            return 1;
        }
    }

    private static async Task<int> HandleClickButtonCommand(string? buttonName)
    {
        buttonName ??= "";
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            if (string.IsNullOrEmpty(buttonName))
                AnsiConsole.MarkupLine("[cyan]Listing all active buttons...[/]");
            else
                AnsiConsole.MarkupLine($"[cyan]Clicking button '{buttonName}'...[/]");
            var result = await client.ClickButtonAsync(buttonName);
            string msg = Markup.Escape(result.Message ?? "");
            if (result.Success)
                AnsiConsole.MarkupLine($"[green]✓[/] {msg}");
            else
                AnsiConsole.MarkupLine($"[red]✗[/] {msg}");
            client.Disconnect();
            return result.Success ? 0 : 1;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {Markup.Escape(ex.Message)}");
            return 1;
        }
    }

    private static async Task<int> HandleToggleUiCommand(string? target)
    {
        target ??= "modmenu";
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            AnsiConsole.MarkupLine($"[cyan]Toggling UI '{target}'...[/]");
            var result = await client.ToggleUiAsync(target);
            string msg = Markup.Escape(result.Message ?? "");
            if (result.Success)
                AnsiConsole.MarkupLine($"[green]✓[/] {msg}");
            else
                AnsiConsole.MarkupLine($"[red]✗[/] {msg}");
            client.Disconnect();
            return result.Success ? 0 : 1;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {Markup.Escape(ex.Message)}");
            return 1;
        }
    }

    private static async Task<int> HandleScanSceneCommand(string? filter)
    {
        filter ??= "";
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            AnsiConsole.MarkupLine($"[cyan]Scanning active MonoBehaviours{(string.IsNullOrEmpty(filter) ? "" : $" (filter: {filter})")}...[/]");
            var result = await client.ScanSceneAsync(filter);
            string msg = result.Message ?? "";
            // Print each line
            foreach (var line in msg.Split('\n', StringSplitOptions.RemoveEmptyEntries))
                AnsiConsole.MarkupLine(Markup.Escape(line));
            client.Disconnect();
            return result.Success ? 0 : 1;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {Markup.Escape(ex.Message)}");
            return 1;
        }
    }

    private static async Task<int> HandleInvokeMethodCommand(string[] args)
    {
        if (args.Length < 2)
        {
            AnsiConsole.MarkupLine("[red]Usage: invoke-method <target> <method>[/]");
            return 1;
        }
        string target = args[0];
        string method = args[1];
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            AnsiConsole.MarkupLine($"[cyan]Invoking {target}.{method}()...[/]");
            var result = await client.InvokeMethodAsync(target, method);
            string msg = Markup.Escape(result.Message ?? "");
            if (result.Success)
                AnsiConsole.MarkupLine($"[green]✓[/] {msg}");
            else
                AnsiConsole.MarkupLine($"[red]✗[/] {msg}");
            client.Disconnect();
            return result.Success ? 0 : 1;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {Markup.Escape(ex.Message)}");
            return 1;
        }
    }

    private static async Task<int> HandleUiTreeCommand(string? selector)
    {
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            AnsiConsole.MarkupLine($"[cyan]Capturing UI tree{(selector != null ? $" (selector: {selector})" : "")}...[/]");
            UiTreeResult result = await client.GetUiTreeAsync(selector);
            if (!result.Success)
            {
                AnsiConsole.MarkupLine($"[red]✗[/] {Markup.Escape(result.Message)}");
                client.Disconnect();
                return 1;
            }
            AnsiConsole.MarkupLine($"[green]✓[/] {result.NodeCount} nodes  ({result.GeneratedAtUtc})");
            PrintUiNode(result.Root, 0);
            client.Disconnect();
            return 0;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Error:[/] {Markup.Escape(ex.Message)}");
            return 1;
        }
    }

    private static void PrintUiNode(UiNode node, int depth)
    {
        string indent = new string(' ', depth * 2);
        string label = string.IsNullOrEmpty(node.Label) ? "" : $" \"{Markup.Escape(node.Label)}\"";
        string interactable = node.Interactable ? " [green]interactive[/]" : "";
        string active = node.Active ? "" : " [grey](inactive)[/]";
        string role = Markup.Escape($"({node.Role})");
        AnsiConsole.MarkupLine($"{indent}[yellow]{Markup.Escape(node.Name)}[/] [grey]{role}[/]{label}{interactable}{active}");
        foreach (UiNode child in node.Children)
            PrintUiNode(child, depth + 1);
    }

    /// <summary>
    /// Full end-to-end demo:
    ///   1. Wait for game connection (main menu)
    ///   2. Screenshot main menu
    ///   3. Click the native DINOForge Mods button → screenshot
    ///   4. Close mod menu (toggle again)
    ///   5. Toggle debug panel (F9 equivalent) → screenshot
    ///   6. Close debug panel
    ///   7. Load AutoSave_1 via ECS LoadRequest
    ///   8. Wait for world to be ready (entity count > 50000)
    ///   9. Dismiss "Press Any Key to Continue"
    ///  10. Status + resources + catalog dump → screenshot
    /// </summary>
    private static async Task<int> HandleDemoCommand()
    {
        AnsiConsole.MarkupLine("[cyan bold]═══════════════════════════════════════[/]");
        AnsiConsole.MarkupLine("[cyan bold]  DINOForge End-to-End Demo            [/]");
        AnsiConsole.MarkupLine("[cyan bold]═══════════════════════════════════════[/]");

        using var client = new GameClient();
        try
        {
            // ── Step 1: Connect ───────────────────────────────────────────────
            AnsiConsole.MarkupLine("[yellow]Step 1:[/] Connecting to game bridge...");
            await client.ConnectAsync();
            AnsiConsole.MarkupLine("[green]✓[/] Connected");

            // ── Step 2: Status at main menu ───────────────────────────────────
            AnsiConsole.MarkupLine("[yellow]Step 2:[/] Checking status (main menu)...");
            var status = await client.StatusAsync();
            AnsiConsole.MarkupLine($"  World ready: {status.WorldReady}  |  Entities: {status.EntityCount}  |  Packs: {status.LoadedPacks.Count}");

            // ── Step 3: Screenshot main menu ──────────────────────────────────
            AnsiConsole.MarkupLine("[yellow]Step 3:[/] Screenshot main menu...");
            string ssMenu = Path.Combine(Path.GetTempPath(), "dinoforge_demo_01_mainmenu.png");
            await client.ScreenshotAsync(ssMenu);
            AnsiConsole.MarkupLine($"[green]✓[/] Screenshot: {Markup.Escape(ssMenu)}");

            // ── Step 4: Click native Mods button ─────────────────────────────
            AnsiConsole.MarkupLine("[yellow]Step 4:[/] Clicking DINOForge_ModsButton (native injected button)...");
            var clickResult = await client.ClickButtonAsync("DINOForge_ModsButton");
            AnsiConsole.MarkupLine(clickResult.Success
                ? $"[green]✓[/] {Markup.Escape(clickResult.Message ?? "")}"
                : $"[red]✗[/] {Markup.Escape(clickResult.Message ?? "")}");

            await Task.Delay(600);
            string ssMods = Path.Combine(Path.GetTempPath(), "dinoforge_demo_02_mods_open.png");
            await client.ScreenshotAsync(ssMods);
            AnsiConsole.MarkupLine($"[green]✓[/] Screenshot (mods menu open): {Markup.Escape(ssMods)}");

            // ── Step 5: Close mod menu ────────────────────────────────────────
            AnsiConsole.MarkupLine("[yellow]Step 5:[/] Closing mod menu (toggle F10 equivalent)...");
            var closeMenu = await client.ToggleUiAsync("modmenu");
            AnsiConsole.MarkupLine(closeMenu.Success
                ? $"[green]✓[/] {Markup.Escape(closeMenu.Message ?? "")}"
                : $"[red]✗[/] {Markup.Escape(closeMenu.Message ?? "")}");

            // ── Step 6: Toggle debug panel (F9 equivalent) ───────────────────
            AnsiConsole.MarkupLine("[yellow]Step 6:[/] Toggling debug panel (F9 equivalent)...");
            await Task.Delay(400);
            var dbgOn = await client.ToggleUiAsync("debug");
            AnsiConsole.MarkupLine(dbgOn.Success
                ? $"[green]✓[/] {Markup.Escape(dbgOn.Message ?? "")}"
                : $"[red]✗[/] {Markup.Escape(dbgOn.Message ?? "")}");

            await Task.Delay(600);
            string ssDebug = Path.Combine(Path.GetTempPath(), "dinoforge_demo_03_debug_open.png");
            await client.ScreenshotAsync(ssDebug);
            AnsiConsole.MarkupLine($"[green]✓[/] Screenshot (debug panel): {Markup.Escape(ssDebug)}");

            // Close debug panel
            await client.ToggleUiAsync("debug");

            // ── Step 7: Load save ─────────────────────────────────────────────
            AnsiConsole.MarkupLine("[yellow]Step 7:[/] Loading AUTOSAVE_1 via ECS bridge...");
            var loadResult = await client.LoadSaveAsync("AUTOSAVE_1");
            AnsiConsole.MarkupLine(loadResult.Success
                ? $"[green]✓[/] {Markup.Escape(loadResult.Message ?? "")}"
                : $"[red]✗[/] {Markup.Escape(loadResult.Message ?? "")}");

            // ── Step 8: Wait for world to load (entities > 50k) ──────────────
            AnsiConsole.MarkupLine("[yellow]Step 8:[/] Waiting for game world to load...");
            int waited = 0;
            GameStatus? worldStatus = null;
            while (waited < 30000)
            {
                await Task.Delay(1500);
                waited += 1500;
                try
                {
                    worldStatus = await client.StatusAsync();
                    if (worldStatus.EntityCount > 50000)
                        break;
                    AnsiConsole.MarkupLine($"  [grey]Entities: {worldStatus.EntityCount} (waiting for >50k)...[/]");
                }
                catch { /* pipe may reconnect */ }
            }
            int finalEntities = worldStatus?.EntityCount ?? 0;
            AnsiConsole.MarkupLine($"[green]✓[/] World loaded: {finalEntities} entities");

            // ── Step 9: Dismiss loading screen ───────────────────────────────
            AnsiConsole.MarkupLine("[yellow]Step 9:[/] Dismissing 'Press Any Key' screen...");
            await Task.Delay(1000);
            var dismissResult = await client.DismissLoadScreenAsync();
            AnsiConsole.MarkupLine(dismissResult.Success
                ? $"[green]✓[/] {Markup.Escape(dismissResult.Message ?? "")}"
                : $"[red]✗[/] {Markup.Escape(dismissResult.Message ?? "")}");

            await Task.Delay(1500);

            // ── Step 10: Gameplay verification ───────────────────────────────
            AnsiConsole.MarkupLine("[yellow]Step 10:[/] Verifying gameplay state...");

            // Status
            var gameStatus = await client.StatusAsync();
            AnsiConsole.MarkupLine($"  [cyan]Entities:[/] {gameStatus.EntityCount}  |  [cyan]World:[/] {gameStatus.WorldName}");
            foreach (var pack in gameStatus.LoadedPacks)
                AnsiConsole.MarkupLine($"    Pack: {Markup.Escape(pack)}");

            // Resources
            var resources = await client.GetResourcesAsync();
            AnsiConsole.MarkupLine($"  [cyan]Resources:[/] Food={resources.Food} Wood={resources.Wood} Stone={resources.Stone} Iron={resources.Iron} Gold={resources.Money}");

            // Catalog
            var catalog = await client.DumpStateAsync();
            AnsiConsole.MarkupLine($"  [cyan]Catalog:[/] {catalog.Units.Count} unit types, {catalog.Buildings.Count} building types, {catalog.Projectiles.Count} projectile types");

            // Final screenshot
            string ssGame = Path.Combine(Path.GetTempPath(), "dinoforge_demo_04_gameplay.png");
            await client.ScreenshotAsync(ssGame);
            AnsiConsole.MarkupLine($"[green]✓[/] Screenshot (gameplay): {Markup.Escape(ssGame)}");

            AnsiConsole.MarkupLine("");
            AnsiConsole.MarkupLine("[green bold]═══════════════════════════════════════[/]");
            AnsiConsole.MarkupLine("[green bold]  Demo complete! Screenshots:           [/]");
            AnsiConsole.MarkupLine($"[green bold]  01[/] {Markup.Escape(ssMenu)}");
            AnsiConsole.MarkupLine($"[green bold]  02[/] {Markup.Escape(ssMods)}");
            AnsiConsole.MarkupLine($"[green bold]  03[/] {Markup.Escape(ssDebug)}");
            AnsiConsole.MarkupLine($"[green bold]  04[/] {Markup.Escape(ssGame)}");
            AnsiConsole.MarkupLine("[green bold]═══════════════════════════════════════[/]");

            client.Disconnect();
            return 0;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]✗ Demo failed:[/] {Markup.Escape(ex.Message)}");
            return 1;
        }
    }

    // ── JSON-output commands (consumed by Python MCP server) ──────────────────

    private static async Task<int> HandleGetStatCommand(string[] args)
    {
        if (args.Length == 0)
        {
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new { error = "Usage: get-stat <sdk_path> [entity_index]" }));
            return 1;
        }
        string sdkPath = args[0];
        int? entityIndex = args.Length > 1 && int.TryParse(args[1], out int idx) ? idx : (int?)null;
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            StatResult result = await client.GetStatAsync(sdkPath, entityIndex);
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new
            {
                sdkPath = result.SdkPath,
                value = result.Value,
                entityCount = result.EntityCount,
                values = result.Values,
                componentType = result.ComponentType,
                fieldName = result.FieldName
            }));
            client.Disconnect();
            return 0;
        }
        catch (Exception ex)
        {
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new { error = ex.Message, sdkPath }));
            return 1;
        }
    }

    private static async Task<int> HandleApplyOverrideCommand(string[] args)
    {
        if (args.Length < 2 || !float.TryParse(args[1], System.Globalization.NumberStyles.Float, System.Globalization.CultureInfo.InvariantCulture, out float value))
        {
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new { error = "Usage: apply-override <sdk_path> <value> [mode] [filter]" }));
            return 1;
        }
        string sdkPath = args[0];
        string? mode = args.Length > 2 ? args[2] : null;
        string? filter = args.Length > 3 ? args[3] : null;
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            OverrideResult result = await client.ApplyOverrideAsync(sdkPath, value, mode, filter);
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new
            {
                success = result.Success,
                modifiedCount = result.ModifiedCount,
                sdkPath = result.SdkPath,
                message = result.Message
            }));
            client.Disconnect();
            return result.Success ? 0 : 1;
        }
        catch (Exception ex)
        {
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new { success = false, error = ex.Message, sdkPath }));
            return 1;
        }
    }

    private static async Task<int> HandleGetComponentMapCommand(string? sdkPathFilter)
    {
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            ComponentMapResult result = await client.GetComponentMapAsync(sdkPathFilter);
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new
            {
                mappings = result.Mappings.Select(m => new
                {
                    sdkPath = m.SdkPath,
                    ecsType = m.EcsType,
                    fieldName = m.FieldName,
                    resolved = m.Resolved,
                    description = m.Description
                })
            }));
            client.Disconnect();
            return 0;
        }
        catch (Exception ex)
        {
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new { error = ex.Message }));
            return 1;
        }
    }

    private static async Task<int> HandleReloadPacksCommand(string? path)
    {
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            ReloadResult result = await client.ReloadPacksAsync(path);
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new
            {
                success = result.Success,
                loadedPacks = result.LoadedPacks,
                errors = result.Errors
            }));
            client.Disconnect();
            return result.Success ? 0 : 1;
        }
        catch (Exception ex)
        {
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new { success = false, error = ex.Message }));
            return 1;
        }
    }

    private static async Task<int> HandleVerifyModCommand(string? packPath)
    {
        if (string.IsNullOrEmpty(packPath))
        {
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new { error = "Usage: verify-mod <pack_path>" }));
            return 1;
        }
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            VerifyResult result = await client.VerifyModAsync(packPath);
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new
            {
                packId = result.PackId,
                loaded = result.Loaded,
                statChanges = result.StatChanges,
                errors = result.Errors,
                entityCount = result.EntityCount
            }));
            client.Disconnect();
            return result.Loaded ? 0 : 1;
        }
        catch (Exception ex)
        {
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new { loaded = false, error = ex.Message, packPath }));
            return 1;
        }
    }

    private static async Task<int> HandleDumpStateJsonCommand(string? category)
    {
        using var client = new GameClient();
        try
        {
            await client.ConnectAsync();
            CatalogSnapshot result = await client.DumpStateAsync(category);
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new
            {
                units = result.Units.Select(e => new { e.InferredId, e.ComponentCount, e.EntityCount, e.Category }),
                buildings = result.Buildings.Select(e => new { e.InferredId, e.ComponentCount, e.EntityCount, e.Category }),
                projectiles = result.Projectiles.Select(e => new { e.InferredId, e.ComponentCount, e.EntityCount, e.Category }),
                other = result.Other.Select(e => new { e.InferredId, e.ComponentCount, e.EntityCount, e.Category })
            }));
            client.Disconnect();
            return 0;
        }
        catch (Exception ex)
        {
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new { error = ex.Message }));
            return 1;
        }
    }
}
