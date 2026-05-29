#nullable enable
using System;
using System.Diagnostics;
using System.Threading;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// Pre/post-flight process cleanup for GameLaunch tests (SPEC-007 / Game Launch Protocol).
/// Mirrors <c>Stop-StrayGameLaunchProcesses</c> in <c>scripts/game/prove-features-gate.ps1</c>.
/// </summary>
internal static class GameLaunchProcessCleanup
{
    internal const string GameProcessBaseName = "Diplomacy is Not an Option";
    private static readonly string[] ProcessNamesToStop =
    {
        GameProcessBaseName,
        "UnityCrashHandler64",
    };

    private const int PostKillVerifyDelayMs = 3_000;

    /// <summary>
    /// Stops stray game-related processes and waits briefly for exit (best-effort).
    /// </summary>
    public static void StopStrayGameProcesses()
    {
        foreach (string processName in ProcessNamesToStop)
        {
            TryKillProcessesByName(processName);
        }

        Thread.Sleep(PostKillVerifyDelayMs);
    }

    private static void TryKillProcessesByName(string processName)
    {
        try
        {
            foreach (Process process in Process.GetProcessesByName(processName))
            {
                using (process)
                {
                    try
                    {
                        if (!process.HasExited)
                        {
                            process.Kill(entireProcessTree: true);
                        }
                    }
                    catch
                    {
                        // Best-effort — another test runner or the OS may own the process.
                    }
                }
            }
        }
        catch
        {
            // GetProcessesByName can throw on some hosts; cleanup remains best-effort.
        }
    }
}
