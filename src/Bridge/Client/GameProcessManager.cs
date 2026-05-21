#nullable enable
using System;
using System.Diagnostics;
using System.IO;
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
    public async Task<bool> LaunchAsync(string? gamePath = null, CancellationToken ct = default)
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
                // Pattern #102: dispose the returned handle to release OS resources.
                // Process keeps running after Dispose — Dispose only releases the parent's handle, not the child process.
                using (var _ = Process.Start(steamInfo)) { }

                // Wait for the game process to appear
                if (await WaitForProcessAsync(timeoutMs: 30000, ct).ConfigureAwait(false))
                    return true;
            }
            catch (OperationCanceledException)
            {
                throw;
            }
            catch // safe-swallow: Steam launch is best-effort; falls through to direct-exe path
            {
                // empty-catch-ok: Steam not available, fall through to direct launch
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
            // Pattern #102: dispose the returned handle to release OS resources.
            // Process keeps running after Dispose — Dispose only releases the parent's handle, not the child process.
            using (var _ = Process.Start(startInfo)) { }

            return await WaitForProcessAsync(timeoutMs: 30000, ct).ConfigureAwait(false);
        }
        catch (OperationCanceledException)
        {
            throw;
        }
        catch // safe-swallow: direct-launch failure reported as bool; caller decides retry/escalate
        {
            // empty-catch-ok: Process.Start failures returned as launch=false to caller
            return false;
        }
    }

    /// <summary>
    /// Kills the game process if it is running.
    /// </summary>
    public async Task KillAsync(CancellationToken ct = default)
    {
        using Process? process = GetGameProcess();
        if (process is null)
            return;

        try
        {
            // netstandard2.0: Kill(bool) does not exist, use Kill() only
            process.Kill();

            // Wait for process exit using polling (netstandard2.0 compatible)
            await WaitForProcessExitAsync(process, ct).ConfigureAwait(false);
        }
        catch (InvalidOperationException) // safe-swallow: Process already exited — Kill() is a no-op
        {
            // empty-catch-ok: race with process exit between GetGameProcess() and Kill()
        }
    }

    /// <summary>
    /// Waits for the game process to exit.
    /// </summary>
    /// <param name="ct">Cancellation token.</param>
    public async Task WaitForExitAsync(CancellationToken ct = default)
    {
        // Check for cancellation immediately
        ct.ThrowIfCancellationRequested();

        // Early return: if no game running or already exited, nothing to wait for
        var process = GetGameProcess();
        if (process is null)
        {
            return;
        }

        try
        {
            if (process.HasExited)
            {
                process.Dispose();
                return;
            }
        }
        catch // safe-swallow: handle may have become invalid mid-check; treat as exited
        {
            // empty-catch-ok: Handle may be invalid; treat as already exited
            process.Dispose();
            return;
        }

        process.Dispose();

        while (!ct.IsCancellationRequested)
        {
            using Process? processInLoop = GetGameProcess();
            if (processInLoop is null)
                return;

            try
            {
                // netstandard2.0: Use polling to wait for process exit
                await WaitForProcessExitAsync(processInLoop, ct).ConfigureAwait(false);
                return;
            }
            catch (OperationCanceledException)
            {
                // Cooperative cancellation: rethrow so caller can distinguish cancel from normal exit.
                throw;
            }
            catch // safe-swallow: handle may have become invalid during HasExited check; re-poll
            {
                // empty-catch-ok: Process handle may have become invalid; re-check on next iteration
                await Task.Delay(500, ct).ConfigureAwait(false);
            }
        }
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
        catch // safe-swallow: enumerating processes can throw if access denied / WMI unavailable; null = "not detected"
        {
            // empty-catch-ok: process enumeration failure treated as "not detected"
            return null;
        }
    }

    private static async Task<bool> WaitForProcessAsync(int timeoutMs, CancellationToken ct = default)
    {
        int elapsed = 0;
        const int pollInterval = 500;

        while (elapsed < timeoutMs)
        {
            ct.ThrowIfCancellationRequested();

            using Process? process = GetGameProcess();
            if (process is not null)
                return true;

            await Task.Delay(pollInterval, ct).ConfigureAwait(false);
            elapsed += pollInterval;
        }

        return false;
    }

    /// <summary>
    /// Waits for a specific process to exit (netstandard2.0 compatible).
    /// </summary>
    private static async Task WaitForProcessExitAsync(Process process, CancellationToken ct = default)
    {
        const int pollIntervalMs = 100;

        while (!ct.IsCancellationRequested)
        {
            if (process.HasExited)
                return;

            await Task.Delay(pollIntervalMs, ct).ConfigureAwait(false);
        }

        ct.ThrowIfCancellationRequested();
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
