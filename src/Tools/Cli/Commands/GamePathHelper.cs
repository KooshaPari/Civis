#nullable enable
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Resolves the DINO game installation path from CLI option, environment, or default.
/// </summary>
internal static class GamePathHelper
{
    /// <summary>
    /// Default game installation path (matches Directory.Build.props).
    /// </summary>
    private const string DefaultGamePath = @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option";

    /// <summary>
    /// Detects the game path using the following priority:
    /// 1. Explicit <paramref name="cliOption"/> (from --game-path)
    /// 2. <c>DINO_GAME_PATH</c> environment variable
    /// 3. Default Steam path
    /// </summary>
    /// <param name="cliOption">Value from --game-path CLI option, or null.</param>
    /// <returns>Resolved game path, or null if not found.</returns>
    public static string? Detect(string? cliOption = null)
    {
        if (!string.IsNullOrWhiteSpace(cliOption))
        {
            if (Directory.Exists(cliOption))
            {
                return cliOption;
            }

            AnsiConsole.MarkupLine($"[red]Error:[/] Specified game path does not exist: {Markup.Escape(cliOption)}");
            return null;
        }

        string? envPath = Environment.GetEnvironmentVariable("DINO_GAME_PATH");
        if (!string.IsNullOrWhiteSpace(envPath))
        {
            if (Directory.Exists(envPath))
            {
                return envPath;
            }

            AnsiConsole.MarkupLine($"[yellow]Warning:[/] DINO_GAME_PATH is set but path does not exist: {Markup.Escape(envPath)}");
        }

        if (Directory.Exists(DefaultGamePath))
        {
            return DefaultGamePath;
        }

        AnsiConsole.MarkupLine("[red]Error:[/] Could not detect game path. Use --game-path or set DINO_GAME_PATH.");
        return null;
    }

    /// <summary>
    /// Gets the game executable path within the game directory.
    /// </summary>
    public static string GetExePath(string gamePath) =>
        Path.Combine(gamePath, "Diplomacy is Not an Option.exe");

    /// <summary>
    /// Gets the BepInEx plugins directory within the game directory.
    /// </summary>
    public static string GetPluginsDir(string gamePath) =>
        Path.Combine(gamePath, "BepInEx", "plugins");

    /// <summary>
    /// Gets the dinoforge_packs directory within the game directory.
    /// </summary>
    public static string GetPacksDir(string gamePath) =>
        Path.Combine(gamePath, "BepInEx", "dinoforge_packs");
}
