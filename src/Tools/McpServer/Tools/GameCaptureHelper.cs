#nullable enable
using System.Diagnostics;
using ScreenRecorderLib;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// Shared screenshot capture logic used by multiple MCP tools.
/// Primary: ScreenRecorderLib (Windows.Graphics.Capture API) — captures per-window DirectX
///          content without requiring focus or window visibility. Non-intrusive.
/// Fallback: ffmpeg gdigrab — GDI desktop capture, requires game window visible on screen.
/// </summary>
internal static class GameCaptureHelper
{
    private const string GameWindowTitle = "Diplomacy is Not an Option";

    /// <summary>
    /// Captures the game window to a PNG file. Returns the output path on success, null on failure.
    /// </summary>
    internal static async Task<string?> CaptureAsync(string outputPath, CancellationToken ct = default)
    {
        // Primary: Windows.Graphics.Capture via ScreenRecorderLib
        // Captures DirectX GPU content per-window, no focus required, non-intrusive
        if (await TryScreenRecorderLibAsync(outputPath, ct).ConfigureAwait(false))
            return outputPath;

        // Fallback: ffmpeg gdigrab (captures desktop GDI composite)
        if (await TryFfmpegGdigrabAsync(outputPath, ct).ConfigureAwait(false))
            return outputPath;

        return null;
    }

    private static async Task<bool> TryScreenRecorderLibAsync(string outputPath, CancellationToken ct)
    {
        try
        {
            var gameProcess = Process.GetProcesses()
                .FirstOrDefault(p => p.MainWindowTitle == GameWindowTitle);
            if (gameProcess?.MainWindowHandle == null || gameProcess.MainWindowHandle == IntPtr.Zero)
                return false;

            var options = new RecorderOptions
            {
                SourceOptions = new SourceOptions
                {
                    RecordingSources =
                    {
                        new WindowRecordingSource(gameProcess.MainWindowHandle)
                        {
                            IsCursorCaptureEnabled = false
                        }
                    }
                }
            };

            using var recorder = Recorder.CreateRecorder(options);

            // v6 API: TakeSnapshot(path) triggers async snapshot; wait via event
            var tcs = new TaskCompletionSource<bool>(TaskCreationOptions.RunContinuationsAsynchronously);
            recorder.OnSnapshotSaved += (_, _) => tcs.TrySetResult(true);
            recorder.OnRecordingFailed += (_, _) => tcs.TrySetResult(false);

            recorder.TakeSnapshot(outputPath);

            using var timeoutCts = CancellationTokenSource.CreateLinkedTokenSource(ct);
            timeoutCts.CancelAfter(TimeSpan.FromSeconds(10));

            try
            {
                await tcs.Task.WaitAsync(timeoutCts.Token).ConfigureAwait(false);
            }
            catch (OperationCanceledException) { }

            return File.Exists(outputPath) && new FileInfo(outputPath).Length > 0;
        }
        catch { return false; }
    }

    private static async Task<bool> TryFfmpegGdigrabAsync(string outputPath, CancellationToken ct)
    {
        try
        {
            string? ffmpeg = FindFfmpeg();
            if (string.IsNullOrEmpty(ffmpeg) || !File.Exists(ffmpeg))
                return false;

            var psi = new ProcessStartInfo
            {
                FileName = ffmpeg,
                Arguments = $"-f gdigrab -framerate 1 -i desktop -frames:v 1 -y \"{outputPath}\"",
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true
            };

            using var proc = Process.Start(psi);
            if (proc == null) return false;

            using var cts = CancellationTokenSource.CreateLinkedTokenSource(ct);
            cts.CancelAfter(TimeSpan.FromSeconds(10));

            try { await proc.WaitForExitAsync(cts.Token).ConfigureAwait(false); }
            catch (OperationCanceledException) { try { proc.Kill(); } catch { } return false; }

            return proc.ExitCode == 0 && File.Exists(outputPath) && new FileInfo(outputPath).Length > 0;
        }
        catch { return false; }
    }

    private static string? FindFfmpeg()
    {
        string[] paths =
        [
            "ffmpeg.exe",
            Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles), "ffmpeg", "bin", "ffmpeg.exe"),
            Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86), "ffmpeg", "bin", "ffmpeg.exe"),
            "C:\\program files\\imagemagick-7.1.0-q16-hdri\\ffmpeg.exe",
            "C:\\Program Files\\ImageMagick\\ffmpeg.exe"
        ];
        return Array.Find(paths, File.Exists);
    }
}
