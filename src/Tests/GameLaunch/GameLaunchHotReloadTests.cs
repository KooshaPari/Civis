#nullable enable
using System.IO;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// GL-007: Hot reload triggers when pack YAML files are modified.
/// Tests verify that touching a pack manifest or unit definition file
/// causes the game to detect the change and reload within 5 seconds.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchHotReloadTests(GameLaunchFixture fixture)
{
    /// <summary>
    /// GL-007: Modifying a pack YAML file triggers reload within 5 seconds.
    /// This test requires the pack directory to be accessible and writable.
    /// </summary>
    [Fact(Skip = "Requires live game with DINOForge loaded")]
    public async Task HotReload_PackYamlChange_TriggersReloadWithin5Seconds()
    {
        // Record initial pack list
        GameStatus initialStatus = await fixture.Client!.StatusAsync();
        var initialPacks = new System.Collections.Generic.List<string>(initialStatus.LoadedPacks);

        initialStatus.LoadedPacks.Should().NotBeEmpty(
            "at least one pack should be loaded at startup");

        // Find the pack directory (assumes packs are in a known location relative to game)
        string? packDir = FindPackDirectory();
        if (packDir == null)
        {
            // Skip test if pack directory not found
            Assert.Fail("Pack directory not found; cannot test hot reload");
            return;
        }

        // Find a YAML file to modify (e.g., units.yaml in the first loaded pack)
        string? yamlFile = FindPackYamlFile(packDir);
        if (yamlFile == null)
        {
            Assert.Fail("No pack YAML files found; cannot test hot reload");
            return;
        }

        // Record current modification time
        var fileInfo = new FileInfo(yamlFile);
        System.DateTime originalMTime = fileInfo.LastWriteTimeUtc;

        // Touch the file (update modification time) to trigger the file watcher
        File.SetLastWriteTimeUtc(yamlFile, System.DateTime.UtcNow);

        // Poll for up to 5 seconds to see if pack reload is detected
        bool reloadDetected = false;
        var sw = System.Diagnostics.Stopwatch.StartNew();

        while (sw.Elapsed.TotalSeconds < 5.0)
        {
            await Task.Delay(500);

            try
            {
                GameStatus polledStatus = await fixture.Client.StatusAsync();

                // Simple heuristic: if entity count changed significantly, reload likely occurred
                // (In a real scenario, we'd have a dedicated reload status endpoint)
                if (polledStatus.EntityCount != initialStatus.EntityCount)
                {
                    reloadDetected = true;
                    break;
                }
            }
            catch
            {
                // Transient error, keep polling
            }
        }

        sw.Stop();

        reloadDetected.Should().BeTrue(
            $"pack hot reload should trigger within 5 seconds of YAML file modification (took {sw.Elapsed.TotalSeconds:F1}s)");
    }

    /// <summary>
    /// Helper: Locate the packs directory on disk.
    /// </summary>
    private static string? FindPackDirectory()
    {
        // Check common relative paths from game directory
        string[] candidates = new[]
        {
            "packs",
            "../../../packs",
            "../../packs",
        };

        foreach (string candidate in candidates)
        {
            if (Directory.Exists(candidate))
                return Path.GetFullPath(candidate);
        }

        // Also check environment variable if set
        string? envPath = System.Environment.GetEnvironmentVariable("DINO_PACKS_PATH");
        if (!string.IsNullOrEmpty(envPath) && Directory.Exists(envPath))
            return envPath;

        return null;
    }

    /// <summary>
    /// Helper: Find a .yaml file in the pack directory.
    /// </summary>
    private static string? FindPackYamlFile(string packDir)
    {
        // Look for any .yaml or .yml file in subdirectories
        try
        {
            string[] yamlFiles = Directory.GetFiles(packDir, "*.yaml", SearchOption.AllDirectories);
            if (yamlFiles.Length > 0)
                return yamlFiles[0];

            yamlFiles = Directory.GetFiles(packDir, "*.yml", SearchOption.AllDirectories);
            if (yamlFiles.Length > 0)
                return yamlFiles[0];
        }
        catch
        {
            // Permissions issue or other error
        }

        return null;
    }
}
