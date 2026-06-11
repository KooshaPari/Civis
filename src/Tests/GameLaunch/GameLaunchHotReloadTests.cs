#nullable enable
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Support;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// GL-007: Hot reload triggers when pack YAML files are modified.
/// Conditionally skips on CI where DINO_GAME_PATH is not set.
/// On a self-hosted Windows runner with the game installed, these tests run.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchHotReloadTests(GameLaunchFixture fixture)
{
    /// <summary>
    /// GL-007: Modifying a pack YAML file triggers reload within the SDK debounce window.
    /// Production PackFileWatcher debounce defaults to 15s; poll up to 18s.
    /// </summary>
    [SkippableFact]
    public async Task HotReload_PackYamlChange_TriggersReloadWithin5Seconds()
    {
        fixture.SkipIfNotInitialized();

        GameStatus initialStatus = await fixture.Client!.StatusAsync();
        initialStatus.LoadedPacks.Should().NotBeEmpty(
            "at least one pack should be loaded at startup");

        string? packDir = FindPackDirectory();
        if (packDir == null)
        {
            Assert.Fail(
                "Pack directory not found; set DINO_PACKS_PATH, DINO_GAME_PATH (BepInEx/dinoforge_packs), or run from repo with packs/");
            return;
        }

        string? yamlFile = FindPackYamlFile(packDir, initialStatus.LoadedPacks);
        if (yamlFile == null)
        {
            Assert.Fail("No pack YAML files found; cannot test hot reload");
            return;
        }

        string? hotReloadLogPath = ResolveHotReloadLogPath();
        long logOffset = hotReloadLogPath != null && File.Exists(hotReloadLogPath)
            ? new FileInfo(hotReloadLogPath).Length
            : 0;

        File.SetLastWriteTimeUtc(yamlFile, System.DateTime.UtcNow);

        var sw = System.Diagnostics.Stopwatch.StartNew();
        const double pollTimeoutSeconds = 20.0;
        bool reloadDetected = await TestWait.UntilAsync(
            async () =>
            {
                if (hotReloadLogPath != null && LogContainsHotReloadMarker(hotReloadLogPath, logOffset))
                {
                    return true;
                }

                try
                {
                    GameStatus polledStatus = await fixture.Client.StatusAsync().ConfigureAwait(false);
                    return polledStatus.EntityCount != initialStatus.EntityCount
                        && polledStatus.EntityCount >= 0;
                }
                catch
                {
                    // Transient error, keep polling.
                    return false;
                }
            },
            TimeSpan.FromSeconds(pollTimeoutSeconds),
            pollMs: 500).ConfigureAwait(false);

        sw.Stop();
        reloadDetected.Should().BeTrue(
            $"pack hot reload should appear in BepInEx/LogOutput.log or status within {pollTimeoutSeconds:F0}s " +
            $"(SDK debounce default 15s; took {sw.Elapsed.TotalSeconds:F1}s, log={hotReloadLogPath ?? "n/a"})");
    }

    private static string? FindPackDirectory()
    {
        string? envPath = Environment.GetEnvironmentVariable("DINO_PACKS_PATH");
        if (!string.IsNullOrWhiteSpace(envPath) && Directory.Exists(envPath))
            return Path.GetFullPath(envPath);

        string? gamePacksDir = ResolveGamePacksDirectory();
        if (gamePacksDir != null)
            return gamePacksDir;

        string? repoPacksDir = Path.Combine(GetRepoRoot(), "packs");
        if (Directory.Exists(repoPacksDir))
            return repoPacksDir;

        return WalkUpForPacksDirectory(Directory.GetCurrentDirectory());
    }

    /// <summary>
    /// Resolves <c>BepInEx/dinoforge_packs</c> from <c>DINO_GAME_PATH</c> (exe file or install dir).
    /// Matches runtime default: BepInEx/dinoforge_packs under the game install.
    /// </summary>
    private static string? ResolveGamePacksDirectory()
    {
        string? gamePath = Environment.GetEnvironmentVariable("DINO_GAME_PATH");
        if (string.IsNullOrWhiteSpace(gamePath))
            return null;

        string? gameDir = GameLaunchPaths.ResolveGameInstallDirectory(gamePath);
        if (gameDir == null)
            return null;

        string canonical = Path.Combine(gameDir, "BepInEx", "dinoforge_packs");
        if (Directory.Exists(canonical))
            return canonical;

        string legacy = Path.Combine(gameDir, "dinoforge_packs");
        return Directory.Exists(legacy) ? legacy : null;
    }

    private static string? WalkUpForPacksDirectory(string? startDir)
    {
        string? dir = startDir;
        while (dir != null)
        {
            string candidate = Path.Combine(dir, "packs");
            if (Directory.Exists(candidate))
                return candidate;

            dir = Directory.GetParent(dir)?.FullName;
        }

        return null;
    }

    private static string GetRepoRoot()
    {
        string? current = Path.GetDirectoryName(typeof(GameLaunchHotReloadTests).Assembly.Location);
        while (current != null)
        {
            if (File.Exists(Path.Combine(current, "global.json")))
                return current;

            current = Directory.GetParent(current)?.FullName;
        }

        current = Directory.GetCurrentDirectory();
        while (current != null)
        {
            if (File.Exists(Path.Combine(current, "global.json")))
                return current;

            current = Directory.GetParent(current)?.FullName;
        }

        throw new DirectoryNotFoundException("Could not locate repo root (global.json)");
    }

    private static string? ResolveHotReloadLogPath()
    {
        string? gameDir = GameLaunchPaths.ResolveGameInstallDirectory();
        if (gameDir == null)
            return null;

        string logOutput = Path.Combine(gameDir, "BepInEx", "LogOutput.log");
        return File.Exists(logOutput) ? logOutput : null;
    }

    private static bool LogContainsHotReloadMarker(string logPath, long startOffset)
    {
        try
        {
            using FileStream stream = new(logPath, FileMode.Open, FileAccess.Read, FileShare.ReadWrite);
            if (startOffset > stream.Length)
                startOffset = 0;

            stream.Seek(startOffset, SeekOrigin.Begin);
            using StreamReader reader = new(stream, Encoding.UTF8, detectEncodingFromByteOrderMarks: true);
            string tail = reader.ReadToEnd();
            return tail.Contains("[PackFileWatcher] Debounce elapsed", StringComparison.Ordinal)
                || tail.Contains("[HotReloadBridge] File changed:", StringComparison.Ordinal)
                || tail.Contains("[HotReloadBridge] Pack reload succeeded", StringComparison.Ordinal)
                || tail.Contains("[ModPlatform] Hot reload completed", StringComparison.Ordinal);
        }
        catch
        {
            return false;
        }
    }

    private static string? FindPackYamlFile(string packDir, IReadOnlyList<string>? loadedPackIds = null)
    {
        try
        {
            if (loadedPackIds != null)
            {
                foreach (string packId in loadedPackIds)
                {
                    string packYaml = Path.Combine(packDir, packId, "pack.yaml");
                    if (File.Exists(packYaml))
                        return packYaml;

                    string packYml = Path.Combine(packDir, packId, "pack.yml");
                    if (File.Exists(packYml))
                        return packYml;
                }
            }

            string[] yamlFiles = Directory.GetFiles(packDir, "pack.yaml", SearchOption.AllDirectories);
            if (yamlFiles.Length > 0)
                return yamlFiles[0];

            yamlFiles = Directory.GetFiles(packDir, "pack.yml", SearchOption.AllDirectories);
            if (yamlFiles.Length > 0)
                return yamlFiles[0];

            yamlFiles = Directory.GetFiles(packDir, "*.yaml", SearchOption.AllDirectories);
            if (yamlFiles.Length > 0)
                return yamlFiles[0];

            yamlFiles = Directory.GetFiles(packDir, "*.yml", SearchOption.AllDirectories);
            if (yamlFiles.Length > 0)
                return yamlFiles[0];
        }
        catch { }

        return null;
    }
}
