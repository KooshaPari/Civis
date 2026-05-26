#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that launches the game from the TEST (isolated) instance directory
/// with a hidden window. The TEST instance bypasses Unity's native single-instance
/// mutex because it lives in a separate directory.
///
/// Usage: Deploy DINOForge.Runtime.dll to the TEST instance first:
///   dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true -p:GameInstallPath="G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST"
/// </summary>
[McpServerToolType]
public sealed class GameLaunchTestTool
{
    /// <summary>
    /// Launches the game from the TEST instance directory with a hidden window.
    /// The TEST instance bypasses Unity's single-instance mutex since it runs
    /// from a different directory than the main game install.
    /// </summary>
    /// <param name="processManager">The game process manager (injected).</param>
    /// <param name="hidden">If true (default), launch with hidden window style. Set false to see the window.</param>
    /// <returns>Launch status including whether TEST instance was found and launched.</returns>
    [McpServerTool(Name = "game_launch_test"), Description(
        "Launch the game from the TEST (isolated) instance with hidden window. " +
        "Bypasses Unity's single-instance mutex by using a separate install directory. " +
        "Deploy Runtime DLL to TEST first: dotnet build -p:DeployToGame=true -p:GameInstallPath=\"...Diplomacy is Not an Option_TEST\"")]
    public static string LaunchTestAsync(
        GameProcessManager processManager,
        [Description("If true (default), launch with hidden window style. False shows the window.")] bool hidden = true)
    {
        if (!processManager.HasTestInstance)
        {
            return GameClientHelper.ToJson(new
            {
                success = false,
                message = "No TEST instance configured. Create a copy of the game at " +
                          "'Diplomacy is Not an Option_TEST' and configure .dino_test_instance_path.",
                testInstancePath = (string?)null,
            });
        }

        string testPath = processManager.TestInstancePath!;
        bool launched = processManager.LaunchTestInstance(hidden);

        return GameClientHelper.ToJson(new
        {
            success = launched,
            message = launched
                ? $"TEST instance launched from: {testPath}"
                : $"Failed to launch TEST instance at: {testPath}",
            testInstancePath = testPath,
            hidden,
        });
    }

    /// <summary>
    /// Returns the current TEST instance configuration (path and availability).
    /// </summary>
    [McpServerTool(Name = "game_test_status"), Description(
        "Returns the TEST instance path and whether it is configured and has the Runtime DLL.")]
    public static string GetTestStatus(GameProcessManager processManager)
    {
        string? testPath = processManager.TestInstancePath;
        bool hasRuntime = false;

        if (!string.IsNullOrEmpty(testPath))
        {
            string dllPath = Path.Combine(testPath, "BepInEx", "plugins", "DINOForge.Runtime.dll");
            hasRuntime = File.Exists(dllPath);
        }

        return GameClientHelper.ToJson(new
        {
            hasTestInstance = processManager.HasTestInstance,
            testInstancePath = testPath,
            hasRuntimeDll = hasRuntime,
            mainGameRunning = processManager.IsRunning,
        });
    }
}
