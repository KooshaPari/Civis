#nullable enable
using System;
using System.IO;

namespace DINOForge.Tools.McpServer;

/// <summary>
/// Path containment for MCP tool inputs (screenshots, pack paths, game executables).
/// Prevents traversal sequences from escaping trusted roots.
/// </summary>
internal static class McpPathSafety
{
    internal static readonly string ScreenshotTempRoot =
        Path.Combine(Path.GetTempPath(), "DINOForge");

    /// <summary>
    /// Resolves a screenshot output path under an allowed root, or throws.
    /// </summary>
    internal static string ResolveScreenshotOutputPath(string candidatePath)
    {
        if (string.IsNullOrWhiteSpace(candidatePath))
            throw new ArgumentException("Screenshot path must be non-empty.", nameof(candidatePath));

        string fullCandidate = Path.GetFullPath(candidatePath);
        foreach (string root in GetScreenshotAllowedRoots())
        {
            if (TryEnsureWithin(root, fullCandidate, out string? safe) && safe is not null)
                return safe;
        }

        throw new UnauthorizedAccessException(
            $"Screenshot path '{candidatePath}' is outside allowed roots (temp, BepInEx, DINOFORGE_SCREENSHOT_ROOT).");
    }

    /// <summary>
    /// Resolves a user pack path under allowed pack roots, or throws.
    /// </summary>
    internal static string ResolvePackPath(string candidatePath)
    {
        if (string.IsNullOrWhiteSpace(candidatePath))
            throw new ArgumentException("Pack path must be non-empty.", nameof(candidatePath));

        string fullCandidate = Path.GetFullPath(candidatePath);
        foreach (string root in GetPackAllowedRoots())
        {
            if (TryEnsureWithin(root, fullCandidate, out string? safe) && safe is not null)
                return safe;
        }

        throw new UnauthorizedAccessException(
            $"Pack path '{candidatePath}' is outside allowed pack roots.");
    }

    /// <summary>
    /// Validates a game executable path: must exist, be a file, and use .exe extension.
    /// </summary>
    internal static string ResolveGameExecutablePath(string candidatePath)
    {
        if (string.IsNullOrWhiteSpace(candidatePath))
            throw new ArgumentException("Game path must be non-empty.", nameof(candidatePath));

        string full = Path.GetFullPath(candidatePath);
        if (!File.Exists(full))
            throw new FileNotFoundException($"Game executable not found: {full}");

        if (!string.Equals(Path.GetExtension(full), ".exe", StringComparison.OrdinalIgnoreCase))
            throw new UnauthorizedAccessException($"Game path must be a .exe file: {full}");

        return full;
    }

    private static string[] GetScreenshotAllowedRoots()
    {
        var roots = new List<string> { ScreenshotTempRoot };
        Directory.CreateDirectory(ScreenshotTempRoot);

        string? bepInEx = TryResolveBepInExRoot();
        if (!string.IsNullOrEmpty(bepInEx))
            roots.Add(bepInEx);

        string? extra = Environment.GetEnvironmentVariable("DINOFORGE_SCREENSHOT_ROOT");
        if (!string.IsNullOrWhiteSpace(extra))
            roots.Add(Path.GetFullPath(extra));

        return roots.ToArray();
    }

    private static string[] GetPackAllowedRoots()
    {
        var roots = new List<string>();
        string? env = Environment.GetEnvironmentVariable("DINOFORGE_PACKS_ROOT");
        if (!string.IsNullOrWhiteSpace(env) && Directory.Exists(env))
            roots.Add(Path.GetFullPath(env));

        string? bepInEx = TryResolveBepInExRoot();
        if (!string.IsNullOrEmpty(bepInEx))
        {
            string packs = Path.Combine(Path.GetDirectoryName(bepInEx) ?? bepInEx, "dinoforge_packs");
            if (Directory.Exists(packs))
                roots.Add(packs);
        }

        string cwdPacks = Path.Combine(Environment.CurrentDirectory, "packs");
        if (Directory.Exists(cwdPacks))
            roots.Add(Path.GetFullPath(cwdPacks));

        return roots.ToArray();
    }

    internal static string? TryResolveBepInExRoot()
    {
        string? gameDir = NativeDepResolver.TryResolve(
            "dino_game_path",
            "DINOFORGE_GAME_PATH",
            [],
            requireFile: false);

        if (!string.IsNullOrEmpty(gameDir))
        {
            string bep = Path.Combine(gameDir, "BepInEx");
            if (Directory.Exists(bep))
                return Path.GetFullPath(bep);
        }

        string? fromEnv = Environment.GetEnvironmentVariable("DINOFORGE_BEPINEX_PATH");
        if (!string.IsNullOrWhiteSpace(fromEnv) && Directory.Exists(fromEnv))
            return Path.GetFullPath(fromEnv);

        return null;
    }

    private static bool TryEnsureWithin(string root, string fullCandidate, out string? resolved)
    {
        resolved = null;
        if (string.IsNullOrWhiteSpace(root))
            return false;

        try
        {
            string rootFull = Path.GetFullPath(root);
            if (!rootFull.EndsWith(Path.DirectorySeparatorChar))
                rootFull += Path.DirectorySeparatorChar;

            string candidateFull = Path.GetFullPath(fullCandidate);
            if (!candidateFull.StartsWith(rootFull, StringComparison.OrdinalIgnoreCase))
                return false;

            resolved = candidateFull;
            return true;
        }
        catch (ArgumentException)
        {
            return false;
        }
        catch (NotSupportedException)
        {
            return false;
        }
    }
}
