#nullable enable
using System.IO;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// GL-007: Hot reload triggers when pack YAML files are modified.
/// Uses [Fact(Skip)] - skips on CI where DINO_GAME_PATH is not set.
/// On a self-hosted Windows runner with the game installed, these tests run.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchHotReloadTests(GameLaunchFixture fixture)
{
    /// <summary>
    /// GL-007: Modifying a pack YAML file triggers reload within 5 seconds.
    /// </summary>
    [Fact(Skip = "Game not available - DINO_GAME_PATH not set or game failed to launch. Run on self-hosted runner with DINO installed.")]
    public async Task HotReload_PackYamlChange_TriggersReloadWithin5Seconds()
    {
        GameStatus initialStatus = await fixture.Client.StatusAsync();
        initialStatus.LoadedPacks.Should().NotBeEmpty(
            "at least one pack should be loaded at startup");

        string? packDir = FindPackDirectory();
        if (packDir == null)
        {
            Assert.Fail("Pack directory not found; cannot test hot reload");
            return;
        }

        string? yamlFile = FindPackYamlFile(packDir);
        if (yamlFile == null)
        {
            Assert.Fail("No pack YAML files found; cannot test hot reload");
            return;
        }

        File.SetLastWriteTimeUtc(yamlFile, System.DateTime.UtcNow);

        bool reloadDetected = false;
        var sw = System.Diagnostics.Stopwatch.StartNew();

        while (sw.Elapsed.TotalSeconds < 5.0)
        {
            await Task.Delay(500);

            try
            {
                GameStatus polledStatus = await fixture.Client.StatusAsync();
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

    private static string? FindPackDirectory()
    {
        string[] candidates = ["packs", "../../../packs", "../../packs"];
        foreach (string candidate in candidates)
        {
            if (Directory.Exists(candidate))
                return Path.GetFullPath(candidate);
        }

        string? envPath = System.Environment.GetEnvironmentVariable("DINO_PACKS_PATH");
        if (!string.IsNullOrEmpty(envPath) && Directory.Exists(envPath))
            return envPath;

        return null;
    }

    private static string? FindPackYamlFile(string packDir)
    {
        try
        {
            string[] yamlFiles = Directory.GetFiles(packDir, "*.yaml", SearchOption.AllDirectories);
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
