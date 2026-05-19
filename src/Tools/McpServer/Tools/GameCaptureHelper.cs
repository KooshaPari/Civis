#nullable enable
using System.Diagnostics;
using System.Runtime.Versioning;
using ScreenCapture.NET;
using ScreenRecorderLib;
using DINOForge.Tools.McpServer.PlayCua;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// Shared screenshot capture logic used by multiple MCP tools.
/// Cascade (in priority order):
/// 1. PlayCUA (playcua-native) — GPU-accelerated, if available
/// 2. bare-cua-native — fast native capture via JSON-RPC
/// 3. Unity ScreenCapture.CaptureScreenshot() via file-signal — works on Parsec/virtual displays
/// 4. ScreenCapture.NET DXGI Desktop Duplication — works for exclusive DX11 fullscreen on physical displays
/// 5. ScreenRecorderLib (Windows.Graphics.Capture) — per-window capture fallback
/// 6. ffmpeg gdigrab — GDI desktop capture (last resort, fails on Parsec)
/// </summary>
internal static class GameCaptureHelper
{
    private const string GameWindowTitle = "Diplomacy is Not an Option";

    private static readonly string BepInExRoot =
        Path.Combine(
            "G:\\SteamLibrary\\steamapps\\common\\Diplomacy is Not an Option",
            "BepInEx");

    private static readonly string ScreenshotRequestFile =
        Path.Combine(BepInExRoot, "dinoforge_screenshot_request.txt");

    private static readonly string ScreenshotDoneFile =
        Path.Combine(BepInExRoot, "dinoforge_screenshot_done.txt");

    /// <summary>
    /// Captures the game window to a PNG file. Returns the output path on success, null on failure.
    /// Cascade: PlayCUA → bare-cua → Unity ScreenCapture → DXGI → ScreenRecorderLib → ffmpeg gdigrab.
    /// </summary>
    internal static async Task<string?> CaptureAsync(string outputPath, CancellationToken ct = default)
    {
        // Primary: playcua-native (GPU-accelerated, optional fast native capture)
        if (await TryPlayCuaAsync(outputPath, ct).ConfigureAwait(false))
            return outputPath;

        // Secondary: bare-cua-native (optional, fast native capture)
        if (await TryBareCuaAsync(outputPath, ct).ConfigureAwait(false))
            return outputPath;

        // Tertiary: Unity ScreenCapture.CaptureScreenshot() via file-signal
        // Works for exclusive fullscreen on Parsec/virtual displays — reads directly from GPU backbuffer.
        if (await TryUnityScreenCaptureAsync(outputPath, ct).ConfigureAwait(false))
            return outputPath;

        // Quaternary: DXGI Desktop Duplication (works for exclusive DX11 on physical displays)
        if (OperatingSystem.IsWindows() && await TryDxgiCaptureAsync(outputPath, ct).ConfigureAwait(false))
            return outputPath;

        // Quinary: Windows.Graphics.Capture via ScreenRecorderLib
        if (await TryScreenRecorderLibAsync(outputPath, ct).ConfigureAwait(false))
            return outputPath;

        // Last resort: ffmpeg gdigrab
        if (await TryFfmpegGdigrabAsync(outputPath, ct).ConfigureAwait(false))
            return outputPath;

        return null;
    }

    /// <summary>
    /// Attempts to use bare-cua-native.exe for screenshot capture (optional fallback).
    /// Looks for binary at: BARE_CUA_NATIVE env var, same directory as this DLL, or hardcoded path.
    /// </summary>
    private static async Task<bool> TryBareCuaAsync(string outputPath, CancellationToken ct)
    {
        try
        {
            string? nativePath = FindBareCuaNative();
            if (string.IsNullOrEmpty(nativePath) || !File.Exists(nativePath))
                return false;

            // Start the native process
            await using var computer = await NativeComputer.StartAsync(nativePath, "warn", ct).ConfigureAwait(false);

            // Capture screenshot by window title
            byte[] pngBytes = await computer.ScreenshotAsync(windowTitle: GameWindowTitle, ct: ct).ConfigureAwait(false);
            if (pngBytes.Length == 0)
                return false;

            // Write to output path
            await File.WriteAllBytesAsync(outputPath, pngBytes, ct).ConfigureAwait(false);
            return File.Exists(outputPath) && new FileInfo(outputPath).Length > 1000;
        }
        catch
        {
            // bare-cua not available or startup failed; fall through to next method
            return false;
        }
    }

    /// <summary>
    /// Attempts to use playcua-native.exe for screenshot capture (optional, GPU-accelerated).
    /// Looks for binary at: PLAYCUA_NATIVE_EXE env var, BARE_CUA_NATIVE fallback, same directory as this DLL, or hardcoded path.
    /// PlayCUA is the primary method when available (tried before bare-cua).
    /// </summary>
    private static async Task<bool> TryPlayCuaAsync(string outputPath, CancellationToken ct)
    {
        try
        {
            string? nativePath = FindPlayCuaNative();
            if (string.IsNullOrEmpty(nativePath) || !File.Exists(nativePath))
                return false;

            // Start the native process
            await using var computer = await NativeComputer.StartAsync(nativePath, "warn", ct).ConfigureAwait(false);

            // Capture screenshot by window title
            byte[] pngBytes = await computer.ScreenshotAsync(windowTitle: GameWindowTitle, ct: ct).ConfigureAwait(false);
            if (pngBytes.Length == 0)
                return false;

            // Write to output path
            await File.WriteAllBytesAsync(outputPath, pngBytes, ct).ConfigureAwait(false);
            return File.Exists(outputPath) && new FileInfo(outputPath).Length > 1000;
        }
        catch
        {
            // playcua not available or startup failed; fall through to next method
            return false;
        }
    }

    /// <summary>
    /// Finds playcua-native.exe by checking (in order):
    /// 1. PLAYCUA_NATIVE_EXE environment variable (CI builds)
    /// 2. BARE_CUA_NATIVE environment variable (legacy fallback)
    /// 3. Same directory as this DLL (bin/)
    /// 4. Hardcoded dev path C:\Users\koosh\bare-cua\target\release\bare-cua-native.exe
    /// </summary>
    private static string? FindPlayCuaNative()
    {
        string[] candidatePaths =
        [
            Environment.GetEnvironmentVariable("PLAYCUA_NATIVE_EXE") ?? string.Empty,
            Environment.GetEnvironmentVariable("BARE_CUA_NATIVE") ?? string.Empty,
            Path.Combine(AppContext.BaseDirectory, "playcua-native.exe"),
            Path.Combine(AppContext.BaseDirectory, "bare-cua-native.exe"),
            "C:\\Users\\koosh\\bare-cua\\target\\release\\bare-cua-native.exe"
        ];

        return candidatePaths.FirstOrDefault(p => !string.IsNullOrEmpty(p) && File.Exists(p));
    }

    /// <summary>
    /// Finds bare-cua-native.exe by checking:
    /// 1. BARE_CUA_NATIVE environment variable
    /// 2. Same directory as this DLL
    /// 3. Hardcoded path C:\Users\koosh\bare-cua\target\release\bare-cua-native.exe
    /// </summary>
    private static string? FindBareCuaNative()
    {
        string[] candidatePaths =
        [
            Environment.GetEnvironmentVariable("BARE_CUA_NATIVE") ?? string.Empty,
            Path.Combine(AppContext.BaseDirectory, "bare-cua-native.exe"),
            "C:\\Users\\koosh\\bare-cua\\target\\release\\bare-cua-native.exe"
        ];

        return candidatePaths.FirstOrDefault(p => !string.IsNullOrEmpty(p) && File.Exists(p));
    }

    /// <summary>
    /// Triggers Unity's ScreenCapture.CaptureScreenshot() by writing a request file.
    /// KeyInputSystem polls for this file ~10x/sec and calls ScreenCapture, then writes done file.
    /// This is the only method that works on Parsec virtual displays.
    /// </summary>
    private static async Task<bool> TryUnityScreenCaptureAsync(string outputPath, CancellationToken ct)
    {
        try
        {
            if (!Directory.Exists(BepInExRoot))
                return false;

            // Clean up any stale done file from previous capture
            if (File.Exists(ScreenshotDoneFile))
                File.Delete(ScreenshotDoneFile);

            // Write request file with desired output path
            await File.WriteAllTextAsync(ScreenshotRequestFile, outputPath, ct).ConfigureAwait(false);

            // Wait for done file to appear (up to 10 seconds)
            using var timeoutCts = CancellationTokenSource.CreateLinkedTokenSource(ct);
            timeoutCts.CancelAfter(TimeSpan.FromSeconds(10));

            try
            {
                while (!timeoutCts.Token.IsCancellationRequested)
                {
                    if (File.Exists(ScreenshotDoneFile))
                    {
                        File.Delete(ScreenshotDoneFile);
                        // Give Unity one more frame to flush the file
                        await Task.Delay(200, ct).ConfigureAwait(false);
                        return File.Exists(outputPath) && new FileInfo(outputPath).Length > 1000;
                    }
                    await Task.Delay(100, timeoutCts.Token).ConfigureAwait(false);
                }
            }
            catch (OperationCanceledException) { } // safe-swallow: timeout expected behavior

            // Clean up request file if game didn't respond
            try { File.Delete(ScreenshotRequestFile); } catch { } // safe-swallow: best-effort cleanup, non-critical
            return false;
        }
        catch { return false; } // safe-swallow: screenshot capture failed, return false to caller
    }

    [SupportedOSPlatform("windows")]
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

                    foreach (var card in cards)
                    {
                        try
                        {
                            var displays = service.GetDisplays(card).ToList();
                            if (displays.Count == 0) continue;

                            using var screenCapture = service.GetScreenCapture(displays[0]);
                            var zone = screenCapture.RegisterCaptureZone(
                                0, 0,
                                screenCapture.Display.Width,
                                screenCapture.Display.Height);

                            screenCapture.CaptureScreen();

                            using (zone.Lock())
                            {
                                if (zone.RawBuffer.Length == 0) continue;

                                // Check if frame is mostly black (Parsec IDD or inactive adapter)
                                var raw = zone.RawBuffer.ToArray();
                                int nonBlack = 0;
                                for (int p = 0; p < Math.Min(raw.Length - 3, 4000); p += 4)
                                {
                                    if (raw[p] > 10 || raw[p + 1] > 10 || raw[p + 2] > 10) nonBlack++;
                                }
                                if (nonBlack < 10) continue; // Skip black frames

                                int width = screenCapture.Display.Width;
                                int height = screenCapture.Display.Height;
                                SaveBgra32AsPng(raw, width, height, outputPath);
                            }

                            if (File.Exists(outputPath) && new FileInfo(outputPath).Length > 1000)
                                return true;
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
    /// Saves raw BGRA32 pixel data as a PNG file using System.Drawing.
    /// </summary>
    [SupportedOSPlatform("windows")]
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
            catch (OperationCanceledException) { } // safe-swallow: timeout expected behavior

            return File.Exists(outputPath) && new FileInfo(outputPath).Length > 0;
        }
        catch { return false; } // safe-swallow: capture failure best-effort fallback
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
            catch (OperationCanceledException) { try { proc.Kill(); } catch { } /* safe-swallow: best-effort kill, non-critical */ return false; }

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
