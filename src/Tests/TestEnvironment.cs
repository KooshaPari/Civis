#nullable enable
using System;
using System.IO;

namespace DINOForge.Tests;

/// <summary>
/// Describes the level of game environment available for testing.
/// Used by <see cref="TestEnvironmentResolver"/> to determine which tests can run.
/// </summary>
public enum TestEnvironment
{
    /// <summary>
    /// No game available. GameFact tests skip with "Game not found".
    /// </summary>
    None = 0,

    /// <summary>
    /// Game is installed but not running. Bridge-dependent tests skip.
    /// </summary>
    InstalledOnly = 1,

    /// <summary>
    /// Game is installed AND the bridge server is connected (ECS world accessible).
    /// </summary>
    GameRunning = 2,

    /// <summary>
    /// Full E2E: game running + Steam overlay + Companion app connected.
    /// </summary>
    FullE2E = 3,
}

/// <summary>
/// Single source of truth for test environment detection.
///
/// Checks (in priority order):
/// 1. DINO_GAME_PATH env var (explicit override, CI/self-hosted)
/// 2. DINO_DOCKER_GAME_STUB env var (Docker/container testing)
/// 3. Auto-detect Steam installation path (local dev)
/// 4. Check if game is actually running (for bridge-connected tests)
///
/// Usage:
/// <code>
/// if (TestEnvironmentResolver.IsGameAvailable)
///     RunGameFactTests();
/// else
///     SkipTests("Game not found at " + TestEnvironmentResolver.GamePath);
/// </code>
/// </summary>
public static class TestEnvironmentResolver
{
    private static readonly string[] KnownSteamGamePaths =
    [
        @"C:\Program Files (x86)\Steam\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
        @"C:\Program Files\Steam\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
        @"D:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
        @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
    ];

    /// <summary>
    /// The resolved path to the game executable, or null if not found.
    /// Priority: DINO_GAME_PATH env var > Docker stub > Steam auto-detect.
    /// </summary>
    public static string? GamePath { get; }

    /// <summary>
    /// True when the game executable was found at any known or configured path.
    /// </summary>
    public static bool IsGameAvailable { get; }

    /// <summary>
    /// True when a Docker/game stub is available (via DINO_DOCKER_GAME_STUB env var).
    /// </summary>
    public static bool IsDockerStub { get; }

    /// <summary>
    /// True when the DINO_BRIDGE_PORT is set and the bridge is expected to be reachable.
    /// </summary>
    public static bool IsBridgeConfigured { get; }

    /// <summary>
    /// The configured bridge port, or the default 7474.
    /// </summary>
    public static int BridgePort { get; }

    /// <summary>
    /// True when Steam is detected on this machine.
    /// </summary>
    public static bool IsSteamAvailable { get; }

    /// <summary>
    /// The detected test environment level.
    /// </summary>
    public static TestEnvironment Current { get; }

    /// <summary>
    /// Human-readable description of the current environment for skip messages.
    /// </summary>
    public static string EnvironmentDescription { get; }

    /// <summary>
    /// Why the game was not found (for skip messages).
    /// </summary>
    public static string? UnavailableReason { get; }

    static TestEnvironmentResolver()
    {
        // Priority 1: Explicit override from environment variable (CI/self-hosted/local-dev)
        string? explicitPath = Environment.GetEnvironmentVariable("DINO_GAME_PATH");
        if (!string.IsNullOrWhiteSpace(explicitPath))
        {
            if (File.Exists(explicitPath))
            {
                GamePath = explicitPath;
            }
            else if (explicitPath.StartsWith("http://") || explicitPath.StartsWith("https://"))
            {
                // Docker stub or remote bridge URL
                GamePath = explicitPath;
                IsDockerStub = true;
            }
            else
            {
                UnavailableReason = $"DINO_GAME_PATH set to '{explicitPath}' but file does not exist";
            }
        }

        // Priority 2: Docker/game stub URL (for CI without game)
        if (GamePath == null)
        {
            string? stubUrl = Environment.GetEnvironmentVariable("DINO_DOCKER_GAME_STUB");
            if (!string.IsNullOrWhiteSpace(stubUrl))
            {
                GamePath = stubUrl;
                IsDockerStub = true;
            }
        }

        // Priority 3: Auto-detect from Steam install locations (local dev)
        if (GamePath == null)
        {
            foreach (string knownPath in KnownSteamGamePaths)
            {
                if (File.Exists(knownPath))
                {
                    GamePath = knownPath;
                    break;
                }
            }

            if (GamePath == null)
            {
                UnavailableReason = "Game not found in any known Steam path. " +
                                   "Set DINO_GAME_PATH env var or install via Steam.";
            }
        }

        IsGameAvailable = !string.IsNullOrEmpty(GamePath) && !IsDockerStub
            ? File.Exists(GamePath)
            : !string.IsNullOrEmpty(GamePath);

        // Bridge configuration
        string? portStr = Environment.GetEnvironmentVariable("DINO_BRIDGE_PORT");
        BridgePort = int.TryParse(portStr, out int port) ? port : 7474;
        IsBridgeConfigured = !string.IsNullOrEmpty(Environment.GetEnvironmentVariable("DINO_BRIDGE_PORT"))
                             || Environment.GetEnvironmentVariable("DINO_GAME_PATH") != null;

        // Steam detection
        IsSteamAvailable = !string.IsNullOrEmpty(
            Environment.GetEnvironmentVariable("SteamAppId"));

        // Determine environment level
        if (!IsGameAvailable)
        {
            Current = TestEnvironment.None;
            EnvironmentDescription = "No game found";
        }
        else if (IsBridgeConfigured)
        {
            Current = TestEnvironment.GameRunning;
            EnvironmentDescription = "Game available + bridge configured";
        }
        else
        {
            Current = TestEnvironment.InstalledOnly;
            EnvironmentDescription = "Game installed but not running";
        }
    }

    /// <summary>
    /// Gets a skip reason string for use in test skip messages.
    /// </summary>
    public static string GetSkipReason(string? requirement = null)
    {
        if (IsGameAvailable)
            return "Game available but bridge not configured. Set DINO_BRIDGE_PORT.";

        string baseReason = UnavailableReason ?? "Game not found";
        if (!string.IsNullOrEmpty(requirement))
            return $"{baseReason}. Requirement: {requirement}. " +
                   "Set DINO_GAME_PATH environment variable to the game executable path.";

        return $"{baseReason}. " +
               "Set DINO_GAME_PATH to the game executable path, " +
               "or DINO_DOCKER_GAME_STUB to a bridge URL for Docker-based testing.";
    }
}
