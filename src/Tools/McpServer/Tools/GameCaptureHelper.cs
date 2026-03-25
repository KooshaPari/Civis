#nullable enable
using System.Diagnostics;
using ScreenCapture.NET;
using ScreenRecorderLib;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// Shared screenshot capture logic used by multiple MCP tools.
/// Primary: ScreenCapture.NET DXGI Desktop Duplication — works for exclusive DX11 fullscreen.
/// Secondary: ScreenRecorderLib (Windows.Graphics.Capture) — per-window capture.
/// Fallback: ffmpeg gdigrab — GDI desktop capture (fails for exclusive fullscreen).
/// </summary>
internal static class GameCaptureHelper
{
    private const string GameWindowTitle = "Diplomacy is Not an Option";

    /// <summary>
    /// Captures the game window to a PNG file. Returns the output path on success, null on failure.
    /// </summary>
    internal static async Task<string?> CaptureAsync(string outputPath, CancellationToken ct = default)
    {
        // Primary: DXGI Desktop Duplication — works for exclusive DX11 fullscreen
        if (await TryDxgiCaptureAsync(outputPath, ct).ConfigureAwait(false))
            return outputPath;

        // Secondary: Windows.Graphics.Capture via ScreenRecorderLib
        if (await TryScreenRecorderLibAsync(outputPath, ct).ConfigureAwait(false))
            return outputPath;

        // Fallback: ffmpeg gdigrab (GDI desktop composite — fails for exclusive fullscreen)
        if (await TryFfmpegGdigrabAsync(outputPath, ct).ConfigureAwait(false))
            return outputPath;

        return null;
    }

    private static async Task<bool> TryDxgiCaptureAsync(string outputPath, CancellationToken ct)
    {
        try
        {
            return await Task.Run(() =>
            {
                try
                {
                    var service = new DX11ScreenCaptureService();
                    var cards = service.GetGraphicsCards().ToList();
                    if (cards.Count == 0) return false;

                    // Try all graphics cards to find the active display
                    foreach (var card in cards)
                    {
                        try
                        {
                            var displays = service.GetDisplays(card).ToList();
                            if (displays.Count == 0) continue;

                            // Use the first (primary) display
                            using var screenCapture = service.GetScreenCapture(displays[0]);
                            var zone = screenCapture.RegisterCaptureZone(
                                0, 0,
                                screenCapture.Display.Width,
                                screenCapture.Display.Height);

                            // Retry logic: some adapters (e.g., Parsec IDD) may return black frames initially
                            // Try up to 3 times with small delays to get a non-black frame
                            const int maxRetries = 3;
                            const int retryDelayMs = 500;

                            for (int attempt = 0; attempt < maxRetries; attempt++)
                            {
                                // Capture frame — blocks up to ~500ms waiting for next frame
                                using var cts2 = new CancellationTokenSource(TimeSpan.FromSeconds(5));
                                screenCapture.CaptureScreen();

                                using (zone.Lock())
                                {
                                    if (zone.RawBuffer.Length == 0)
                                    {
                                        if (attempt < maxRetries - 1)
                                        {
                                            System.Threading.Thread.Sleep(retryDelayMs);
                                            continue;
                                        }
                                        break;
                                    }

                                    // Check if frame is mostly black (game may not be ready yet)
                                    if (IsFrameMostlyBlack(zone.RawBuffer.ToArray()))
                                    {
                                        if (attempt < maxRetries - 1)
                                        {
                                            System.Threading.Thread.Sleep(retryDelayMs);
                                            continue;
                                        }
                                        // Last attempt was also black; skip to next card
                                        break;
                                    }

                                    // Frame is valid (has content); save and return success
                                    int width = screenCapture.Display.Width;
                                    int height = screenCapture.Display.Height;
                                    byte[] rawPixels = zone.RawBuffer.ToArray();

                                    SaveBgra32AsPng(rawPixels, width, height, outputPath);

                                    if (File.Exists(outputPath) && new FileInfo(outputPath).Length > 1000)
                                        return true;

                                    // File was created but is too small; try next card
                                    break;
                                }
                            }
                        }
                        catch { continue; }
                    }
                    return false;
                }
                catch { return false; }
            }, ct).ConfigureAwait(false);
        }
        catch { return false; }
    }

    /// <summary>
    /// Checks if a BGRA32 frame buffer is mostly black (indicates adapter not rendering yet).
    /// Samples first 1000 pixels; if > 95% have RGB values &lt; 10, considers it "black".
    /// </summary>
    private static bool IsFrameMostlyBlack(byte[] bgra32)
    {
        if (bgra32.Length < 4000) return true; // Not enough data

        int sampleCount = 0;
        int blackPixels = 0;
        const int sampleSize = 1000;

        for (int i = 0; i < Math.Min(bgra32.Length, sampleSize * 4); i += 4)
        {
            byte b = bgra32[i];
            byte g = bgra32[i + 1];
            byte r = bgra32[i + 2];
            // Ignore alpha (bgra32[i + 3])

            if (r < 10 && g < 10 && b < 10)
                blackPixels++;

            sampleCount++;
        }

        if (sampleCount == 0) return true;

        double blackRatio = (double)blackPixels / sampleCount;
        return blackRatio > 0.95; // > 95% black = mostly black
    }

    /// <summary>
    /// Saves raw BGRA32 pixel data as a PNG file using System.Drawing.
    /// </summary>
    private static void SaveBgra32AsPng(byte[] bgra32, int width, int height, string outputPath)
    {
        using var bmp = new System.Drawing.Bitmap(width, height, System.Drawing.Imaging.PixelFormat.Format32bppArgb);
        var bmpData = bmp.LockBits(
            new System.Drawing.Rectangle(0, 0, width, height),
            System.Drawing.Imaging.ImageLockMode.WriteOnly,
            System.Drawing.Imaging.PixelFormat.Format32bppArgb);
        try
        {
            System.Runtime.InteropServices.Marshal.Copy(bgra32, 0, bmpData.Scan0, bgra32.Length);
        }
        finally
        {
            bmp.UnlockBits(bmpData);
        }
        bmp.Save(outputPath, System.Drawing.Imaging.ImageFormat.Png);
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
