#nullable enable
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;

namespace DINOForge.Bridge.Client;

/// <summary>
/// Manages the Diplomacy is Not an Option game process lifecycle.
/// Provides methods to launch, monitor, and terminate the game.
/// </summary>
public sealed class GameProcessManager
{
    private const int SteamAppId = 1272320;
    private const string ProcessName = "Diplomacy is Not an Option";

    private static readonly string[] DefaultSteamPaths =
    [
        @"C:\Program Files (x86)\Steam\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
        @"C:\Program Files\Steam\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
        @"D:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
    ];

    /// <summary>
    /// Gets whether the game process is currently running.
    /// </summary>
    public bool IsRunning => GetGameProcess() is not null;

    /// <summary>
    /// Gets the process ID of the running game, or null if not running.
    /// </summary>
    /// <returns>The process ID, or null if the game is not running.</returns>
    public int? GetProcessId()
    {
        using Process? process = GetGameProcess();
        return process?.Id;
    }

    /// <summary>
    /// Launches the game, preferring Steam and falling back to a direct exe launch.
    /// </summary>
    /// <param name="gamePath">
    /// Optional path to the game executable. If null, attempts Steam launch
    /// followed by probing common install locations.
    /// </param>
    /// <returns>True if the game process was detected after launch.</returns>
    public async Task<bool> LaunchAsync(string? gamePath = null)
    {
        if (IsRunning)
            return true;

        // Try Steam launch first (works cross-platform where Steam is installed)
        if (gamePath is null)
        {
            try
            {
                ProcessStartInfo steamInfo = new()
                {
                    FileName = $"steam://rungameid/{SteamAppId}",
                    UseShellExecute = true
                };
                Process.Start(steamInfo);

                // Wait for the game process to appear
                if (await WaitForProcessAsync(timeoutMs: 30000).ConfigureAwait(false))
                    return true;
            }
            catch
            {
                // Steam not available, fall through to direct launch
            }
        }

        // Resolve exe path
        string? exePath = gamePath ?? FindGameExe();
        if (exePath is null)
            return false;

        if (!File.Exists(exePath))
            return false;

        try
        {
            ProcessStartInfo startInfo = new()
            {
                FileName = exePath,
                UseShellExecute = true
            };
            Process.Start(startInfo);

            return await WaitForProcessAsync(timeoutMs: 30000).ConfigureAwait(false);
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Kills the game process if it is running.
    /// </summary>
    public async Task KillAsync()
    {
        using Process? process = GetGameProcess();
        if (process is null)
            return;

        try
        {
            process.Kill(entireProcessTree: true);
            await process.WaitForExitAsync().ConfigureAwait(false);
        }
        catch (InvalidOperationException)
        {
            // Process already exited
        }
    }

    /// <summary>
    /// Waits for the game process to exit.
    /// </summary>
    /// <param name="ct">Cancellation token.</param>
    public async Task WaitForExitAsync(CancellationToken ct = default)
    {
        while (!ct.IsCancellationRequested)
        {
            using Process? process = GetGameProcess();
            if (process is null)
                return;

            try
            {
                await process.WaitForExitAsync(ct).ConfigureAwait(false);
                return;
            }
            catch (OperationCanceledException)
            {
                throw;
            }
            catch
            {
                // Process handle may have become invalid; re-check
                await Task.Delay(500, ct).ConfigureAwait(false);
            }
        }

        ct.ThrowIfCancellationRequested();
    }

    private static Process? GetGameProcess()
    {
        try
        {
            Process[] processes = Process.GetProcessesByName(ProcessName);
            if (processes.Length == 0)
                return null;

            // Return the first, dispose the rest
            Process result = processes[0];
            for (int i = 1; i < processes.Length; i++)
                processes[i].Dispose();
            return result;
        }
        catch
        {
            return null;
        }
    }

    private static async Task<bool> WaitForProcessAsync(int timeoutMs)
    {
        int elapsed = 0;
        const int pollInterval = 500;

        while (elapsed < timeoutMs)
        {
            using Process? process = GetGameProcess();
            if (process is not null)
                return true;

            await Task.Delay(pollInterval).ConfigureAwait(false);
            elapsed += pollInterval;
        }

        return false;
    }

    private static string? FindGameExe()
    {
        foreach (string path in DefaultSteamPaths)
        {
            if (File.Exists(path))
                return path;
        }

        return null;
    }
}
