#nullable enable
using System;
using System.Threading;
using System.Threading.Tasks;
using Xunit;
using Xunit.Abstractions;

namespace DINOForge.Tests.Xunit;

/// <summary>
/// Marks a test as requiring the DINO game to be available.
///
/// Skips automatically when:
/// - DINO_GAME_PATH is not set AND no game is found in known Steam paths
/// - Game executable does not exist at the configured path
///
/// Does NOT skip when the game IS available — this is the key difference
/// from [Fact(Skip = "...")]. On a self-hosted runner with DINO_GAME_PATH
/// set, these tests WILL run.
///
/// Usage:
/// <code>
/// [GameFact]                                    // Game must be installed
/// public void TestRequiresGame() { ... }
///
/// [GameFact(RequiresBridge = true)]             // Game + bridge must be connected
/// public async Task TestRequiresBridge() { ... }
///
/// [GameFact(RequiresCatalog = true)]            // VanillaCatalog must be built
/// public void TestRequiresCatalog() { ... }
/// </code>
///
/// For CI environments (ubuntu-latest without game):
/// - Tests marked [GameFact] will skip with a clear message
/// - Self-hosted runner with DINO_GAME_PATH set will run these tests
/// </summary>
[AttributeUsage(AttributeTargets.Method, AllowMultiple = false, Inherited = true)]
public sealed class GameFactAttribute : FactAttribute
{
    /// <summary>
    /// If true, the test requires the bridge to be connected (game running with DINOForge loaded).
    /// Default: false — game just needs to be installed.
    /// </summary>
    public bool RequiresBridge { get; init; }

    /// <summary>
    /// If true, the test requires VanillaCatalog to be built from the game binary.
    /// Default: false.
    /// </summary>
    public bool RequiresCatalog { get; init; }

    /// <summary>
    /// If true, the test requires Steam to be available (for UI automation tests).
    /// Default: false.
    /// </summary>
    public bool RequiresSteam { get; init; }

    /// <summary>
    /// If true, the test requires the Companion app to be running.
    /// Default: false.
    /// </summary>
    public bool RequiresCompanion { get; init; }

    public override Task<RunSummary> RunAsync(
        IMethodInfo method,
        IReflectionAssemblyInfo assembly,
        IAttributeInfo factAttribute,
        IMessageBus messageBus,
        ExceptionAggregator aggregator,
        CancellationTokenSource cancellationTokenSource)
    {
        // Check 1: Game path available
        if (!TestEnvironmentResolver.IsGameAvailable)
        {
            string reason = GetSkipReason();
            return Task.FromResult(SkipTest(method, messageBus, reason));
        }

        // Check 2: Bridge required
        if (RequiresBridge && !TestEnvironmentResolver.IsBridgeConfigured)
        {
            return Task.FromResult(SkipTest(method, messageBus,
                "Bridge not configured. Set DINO_BRIDGE_PORT env var when game is running."));
        }

        // Check 3: Catalog required
        if (RequiresCatalog)
        {
            bool catalogBuilt = CheckVanillaCatalog();
            if (!catalogBuilt)
            {
                return Task.FromResult(SkipTest(method, messageBus,
                    "VanillaCatalog not built. " +
                    "Run: dotnet run --project src/Tools/DumpTools -- build-catalog. " +
                    "Tests that require the catalog will skip until it is built."));
            }
        }

        // Check 4: Steam required
        if (RequiresSteam && !TestEnvironmentResolver.IsSteamAvailable)
        {
            return Task.FromResult(SkipTest(method, messageBus,
                "Steam not available. Set SteamAppId env var."));
        }

        // Check 5: Companion required
        if (RequiresCompanion && !IsCompanionAvailable())
        {
            return Task.FromResult(SkipTest(method, messageBus,
                "Companion app not running. Set COMPANION_PORT env var."));
        }

        // All checks passed — run the test
        return base.RunAsync(method, assembly, factAttribute, messageBus, aggregator, cancellationTokenSource);
    }

    private static RunSummary SkipTest(IMethodInfo method, IMessageBus messageBus, string reason)
    {
        var test = new XunitTest(method, method.Name);
        var skipMessage = new XunitSkipReason(test, reason);
        messageBus.QueueMessage(skipMessage);
        return new RunSummary { Skipped = 1 };
    }

    private static string GetSkipReason()
    {
        if (TestEnvironmentResolver.IsDockerStub)
        {
            return "Docker/game stub configured at: " + (TestEnvironmentResolver.GamePath ?? "unknown");
        }

        return "Game not found. Set DINO_GAME_PATH to the Diplomacy is Not an Option executable, " +
               "or DINO_DOCKER_GAME_STUB to a bridge URL. " +
               "Current: " + TestEnvironmentResolver.EnvironmentDescription;
    }

    private static bool CheckVanillaCatalog()
    {
        try
        {
            var catalogType = Type.GetType("DINOForge.Runtime.VanillaCatalog, DINOForge.Runtime");
            if (catalogType == null)
                return false;

            var prop = catalogType.GetProperty("IsBuilt");
            if (prop == null)
                return false;

            var value = prop.GetValue(null);
            return value is bool built && built;
        }
        catch
        {
            return false;
        }
    }

    private static bool IsCompanionAvailable()
    {
        try
        {
            string? companionPort = Environment.GetEnvironmentVariable("COMPANION_PORT");
            return !string.IsNullOrEmpty(companionPort) && int.TryParse(companionPort, out _);
        }
        catch
        {
            return false;
        }
    }
}
