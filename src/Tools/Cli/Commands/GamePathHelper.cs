#nullable enable
namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Resolves the DINO game install path from caller override, environment, or default.
/// </summary>
internal static class GamePathHelper
{
    /// <summary>
    /// Default DINO Steam install path on Windows.
    /// </summary>
    public const string DefaultGamePath = @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option";

    /// <summary>
    /// Environment variable used to override the default DINO install path.
    /// </summary>
    public const string EnvVarName = "DINO_GAME_PATH";

    /// <summary>
    /// Resolves the DINO install root.
    /// </summary>
    /// <param name="overridePath">An explicit override (e.g. <c>--game-path</c>).</param>
    /// <returns>The absolute path of the DINO install root.</returns>
    public static string Detect(string? overridePath = null)
    {
        if (!string.IsNullOrWhiteSpace(overridePath))
        {
            return overridePath;
        }

        string? env = Environment.GetEnvironmentVariable(EnvVarName);
        if (!string.IsNullOrWhiteSpace(env))
        {
            return env;
        }

        return DefaultGamePath;
    }

    /// <summary>
    /// Returns the path to the DINO executable.
    /// </summary>
    public static string GetExePath(string gamePath) =>
        Path.Combine(gamePath, "Diplomacy is Not an Option.exe");

    /// <summary>
    /// Returns the path to the BepInEx plugins directory.
    /// </summary>
    public static string GetPluginsDir(string gamePath) =>
        Path.Combine(gamePath, "BepInEx", "plugins");

    /// <summary>
    /// Returns the path to the DINOForge packs directory inside BepInEx.
    /// </summary>
    public static string GetPacksDir(string gamePath) =>
        Path.Combine(gamePath, "BepInEx", "dinoforge_packs");
}
