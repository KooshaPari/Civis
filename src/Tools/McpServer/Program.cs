#nullable enable
using DINOForge.Bridge.Client;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;

namespace DINOForge.Tools.McpServer;

/// <summary>
/// Entry point for the DINOForge MCP server.
/// Runs as a stdio-based MCP process that Claude Code connects to,
/// bridging game operations through the named pipe GameClient.
/// </summary>
public static class Program
{
    public static async Task Main(string[] args)
    {
        IHostBuilder builder = Host.CreateDefaultBuilder(args)
            .ConfigureServices((context, services) =>
            {
                // Register GameClient and GameProcessManager as singletons
                services.AddSingleton<GameClientOptions>();
                services.AddSingleton<GameClient>();
                services.AddSingleton<GameProcessManager>();

                // Register the MCP server with stdio transport and all tools
                services.AddMcpServer(options =>
                {
                    options.ServerInfo = new()
                    {
                        Name = "dinoforge",
                        Version = "0.1.0"
                    };
                })
                .WithStdioServerTransport()
                .WithTools<Tools.GameLaunchTool>()
                .WithTools<Tools.GameStatusTool>()
                .WithTools<Tools.GameWaitForWorldTool>()
                .WithTools<Tools.GameQueryEntitiesTool>()
                .WithTools<Tools.GameGetStatTool>()
                .WithTools<Tools.GameApplyOverrideTool>()
                .WithTools<Tools.GameReloadPacksTool>()
                .WithTools<Tools.GameDumpStateTool>()
                .WithTools<Tools.GameScreenshotTool>()
                .WithTools<Tools.GameAnalyzeScreenTool>()
                .WithTools<Tools.GameInputTool>()
                .WithTools<Tools.GameVerifyModTool>()
                .WithTools<Tools.GameGetResourcesTool>()
                .WithTools<Tools.GameGetComponentMapTool>()
                .WithTools<Tools.GameUIAutomationTool>()
                .WithTools<Tools.GameWaitAndScreenshotTool>()
                .WithTools<Tools.GameNavigateTool>();
            });

        await builder.Build().RunAsync().ConfigureAwait(false);
    }
}
