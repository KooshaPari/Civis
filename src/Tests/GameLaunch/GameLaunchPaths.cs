#nullable enable
using System;
using System.IO;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// Resolves game install paths from <c>DINO_GAME_PATH</c> (exe file or install directory).
/// Never treats <c>steamapps/common</c> or its parent as the install root.
/// </summary>
internal static class GameLaunchPaths
{
    public const string GameExecutableName = "Diplomacy is Not an Option.exe";

    /// <summary>
    /// Directory containing <see cref="GameExecutableName"/> and <c>BepInEx/</c>, or null if unresolved.
    /// </summary>
    public static string? ResolveGameInstallDirectory(string? gamePath = null)
    {
        gamePath ??= Environment.GetEnvironmentVariable("DINO_GAME_PATH");
        if (string.IsNullOrWhiteSpace(gamePath))
        {
            return null;
        }

        string? exePath = ResolveGameExecutablePath(gamePath);
        if (exePath is not null)
        {
            return Path.GetDirectoryName(exePath);
        }

        if (!Directory.Exists(gamePath))
        {
            return null;
        }

        // Directory without the exe — reject steam library roots (not the game install).
        string fullDir = Path.GetFullPath(gamePath);
        string leaf = Path.GetFileName(fullDir.TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar));
        if (string.Equals(leaf, "common", StringComparison.OrdinalIgnoreCase)
            || string.Equals(leaf, "steamapps", StringComparison.OrdinalIgnoreCase))
        {
            return null;
        }

        return fullDir;
    }

    /// <summary>
    /// Resolves <paramref name="gamePath"/> to the game executable when possible.
    /// </summary>
    public static string? ResolveGameExecutablePath(string gamePath)
    {
        if (File.Exists(gamePath))
        {
            return gamePath;
        }

        if (!Directory.Exists(gamePath))
        {
            return null;
        }

        string exeInDir = Path.Combine(gamePath, GameExecutableName);
        return File.Exists(exeInDir) ? exeInDir : null;
    }

    public static string? ResolveBepInExPath(string? gamePath = null)
    {
        string? installDir = ResolveGameInstallDirectory(gamePath);
        return installDir is null ? null : Path.Combine(installDir, "BepInEx");
    }
}
