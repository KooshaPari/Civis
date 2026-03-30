#nullable enable
using DINOForge.Bridge.Client;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

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
            .ConfigureLogging(logging =>
            {
                // MCP stdio transport uses stdout for JSON-RPC messages.
                // All logging MUST go to stderr to avoid corrupting the JSON-RPC stream.
                logging.ClearProviders();
                logging.AddProvider(new StderrLoggerProvider());
            })
            .ConfigureServices((context, services) =>
            {
                // Register GameClient and GameProcessManager as singletons
                services.AddSingleton<GameClientOptions>();
                services.AddSingleton<GameClient>();
                services.AddSingleton<GameProcessManager>(); // Owned by this file; wraps Process.Start()

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
                .WithTools<Tools.GameLaunchTestTool>()
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

/// <summary>Writes all log output to stderr so stdout remains clean for JSON-RPC.</summary>
file sealed class StderrLoggerProvider : ILoggerProvider
{
    public ILogger CreateLogger(string categoryName) => new StderrLogger(categoryName);
    public void Dispose() { }
}

file sealed class StderrLogger(string category) : ILogger
{
    public IDisposable? BeginScope<TState>(TState state) where TState : notnull => null;
    public bool IsEnabled(LogLevel logLevel) => logLevel >= LogLevel.Warning;
    public void Log<TState>(LogLevel logLevel, EventId eventId, TState state, Exception? exception, Func<TState, Exception?, string> formatter)
    {
        if (!IsEnabled(logLevel)) return;
        Console.Error.WriteLine($"[{logLevel}] {category}: {formatter(state, exception)}");
        if (exception != null) Console.Error.WriteLine(exception);
    }
}
