#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;

namespace DINOForge.Tools.McpServer;

/// <summary>
/// Unified resolver for native binary + game-path discovery.
/// Walks: env var -> installer-shipped paths.json -> hardcoded fallback -> throws.
/// </summary>
/// <remarks>
/// Replaces three ad-hoc TODO sites that each had their own bespoke fallback chain:
///   - GameCaptureHelper.ResolveBepInExRoot (DINO game path)
///   - GameCaptureHelper.ResolveBareCuaPath (bare-cua-native)
///   - isolation_layer.PlayCUABackend._resolve_playcua_path (Python sibling, see native_dep_resolver.py)
///
/// The installer (when shipped) writes a JSON dictionary to
/// %ProgramData%\DINOForge\paths.json on Windows or /etc/dinoforge/paths.json on Linux,
/// mapping logical keys (e.g. "dino_game_path") to absolute paths discovered at install time.
/// </remarks>
public static class NativeDepResolver
{
    private const string InstallerPathsFile = "paths.json";
    private const string InstallerSubdir = "DINOForge";

    /// <summary>
    /// Resolve a native dependency by key.
    /// </summary>
    /// <param name="key">Logical key in installer paths.json (e.g. "dino_game_path", "bare_cua_native", "playcua_native").</param>
    /// <param name="envVar">Environment variable name that overrides everything (e.g. "DINOFORGE_GAME_PATH").</param>
    /// <param name="hardcodedFallbacks">Last-resort paths tried in order. Used for dev convenience only.</param>
    /// <param name="description">Human-readable description used in the error message.</param>
    /// <param name="requireFile">If true, candidate must be a file. If false (game-path mode), a directory is acceptable.</param>
    /// <returns>An absolute path that exists.</returns>
    /// <exception cref="FileNotFoundException">Thrown when no candidate exists. The message lists every probe attempt.</exception>
    public static string Resolve(
        string key,
        string envVar,
        IEnumerable<string> hardcodedFallbacks,
        string description,
        bool requireFile = true)
    {
        var fallbacks = hardcodedFallbacks?.ToArray() ?? Array.Empty<string>();

        // 1. Env var
        var fromEnv = Environment.GetEnvironmentVariable(envVar);
        if (!string.IsNullOrWhiteSpace(fromEnv) && Exists(fromEnv, requireFile))
            return fromEnv;

        // 2. Installer paths.json
        var installerPaths = TryLoadInstallerPaths();
        string? fromInstaller = null;
        if (installerPaths != null && installerPaths.TryGetValue(key, out var installerVal))
        {
            fromInstaller = installerVal;
            if (!string.IsNullOrWhiteSpace(fromInstaller) && Exists(fromInstaller, requireFile))
                return fromInstaller;
        }

        // 3. Hardcoded fallbacks
        foreach (var fallback in fallbacks)
        {
            if (!string.IsNullOrWhiteSpace(fallback) && Exists(fallback, requireFile))
                return fallback;
        }

        // 4. Loud error
        var fallbackList = fallbacks.Length > 0 ? string.Join(", ", fallbacks) : "<none>";
        throw new FileNotFoundException(
            $"Native dependency '{description}' (key={key}) not found. Tried:\n" +
            $"  - Env var ${envVar}: {(string.IsNullOrEmpty(fromEnv) ? "<unset>" : fromEnv)}\n" +
            $"  - Installer paths.json: {fromInstaller ?? "<not in paths.json>"}\n" +
            $"  - Hardcoded fallbacks: [{fallbackList}]\n" +
            $"Set ${envVar} to override, or run the DINOForge installer to populate paths.json."
        );
    }

    /// <summary>
    /// Try-resolve variant. Returns null instead of throwing when nothing matches.
    /// Useful for "best-effort" call sites (e.g. optional fallback binaries).
    /// </summary>
    public static string? TryResolve(
        string key,
        string envVar,
        IEnumerable<string> hardcodedFallbacks,
        bool requireFile = true)
    {
        try
        {
            return Resolve(key, envVar, hardcodedFallbacks, key, requireFile);
        }
        catch (FileNotFoundException)
        {
            return null;
        }
    }

    private static bool Exists(string path, bool requireFile)
    {
        return requireFile ? File.Exists(path) : (File.Exists(path) || Directory.Exists(path));
    }

    /// <summary>
    /// Locates and parses the installer paths.json. Returns null when the file
    /// is missing, unreadable, or malformed (the resolver continues to fallbacks).
    /// </summary>
    internal static string GetInstallerPathsFilePath()
    {
        var programData = Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData);
        if (string.IsNullOrEmpty(programData))
            programData = OperatingSystem.IsWindows() ? @"C:\ProgramData" : "/etc";
        return Path.Combine(programData, InstallerSubdir, InstallerPathsFile);
    }

    private static Dictionary<string, string>? TryLoadInstallerPaths()
    {
        var pathsFile = GetInstallerPathsFilePath();
        if (!File.Exists(pathsFile)) return null;

        try
        {
            var json = File.ReadAllText(pathsFile, Encoding.UTF8);
            return JsonSerializer.Deserialize<Dictionary<string, string>>(json);
        }
        catch
        {
            return null;
        }
    }
}
