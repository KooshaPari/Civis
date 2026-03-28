#nullable enable
using System.Diagnostics;
using System.Runtime.Versioning;
using System.CommandLine;
using DINOForge.Tools.Cli.Commands;
using Spectre.Console;
using ScreenRecorderLibRec = ScreenRecorderLib;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Records a video clip of the running game window.
/// </summary>
[SupportedOSPlatform("windows")]
internal static partial class RecordCommand
{
    private const string GameWindowTitle = "Diplomacy is Not an Option";
    private const int DefaultDuration = 6;
    private const int DefaultWidth = 1280;
    private const int DefaultHeight = 800;
    private const int DefaultFramerate = 30;
    private const int DefaultQuality = 80;

    /// <summary>
    /// Creates the <c>record</c> command.
    /// </summary>
    public static Command Create()
    {
        var outputOpt = new Option<string>("--output")
        {
            Description = "Output file path for the recording (MP4, required)"
        };

        var durationOpt = new Option<int>("--duration")
        {
            Description = $"Recording duration in seconds (default: {DefaultDuration})"
        };

        var widthOpt = new Option<int>("--width")
        {
            Description = $"Output video width (default: {DefaultWidth})"
        };

        var heightOpt = new Option<int>("--height")
        {
            Description = $"Output video height (default: {DefaultHeight})"
        };

        var framerateOpt = new Option<int>("--framerate")
        {
            Description = $"Output video framerate (default: {DefaultFramerate})"
        };

        var qualityOpt = new Option<int>("--quality")
        {
            Description = $"Output video quality 1-100 (default: {DefaultQuality})"
        };

        Option<string> formatOpt = CommandOutput.CreateFormatOption();

        Command command = new("record", "Record a video clip of the game window");
        command.Add(outputOpt);
        command.Add(durationOpt);
        command.Add(widthOpt);
        command.Add(heightOpt);
        command.Add(framerateOpt);
        command.Add(qualityOpt);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);
            string? output = parseResult.GetValue(outputOpt);
            int duration = parseResult.GetValue(durationOpt);
            int width = parseResult.GetValue(widthOpt);
            int height = parseResult.GetValue(heightOpt);
            int framerate = parseResult.GetValue(framerateOpt);
            int quality = parseResult.GetValue(qualityOpt);

            // Apply defaults
            duration = duration > 0 ? duration : DefaultDuration;
            width = width > 0 ? width : DefaultWidth;
            height = height > 0 ? height : DefaultHeight;
            framerate = framerate > 0 ? framerate : DefaultFramerate;
            quality = quality > 0 ? quality : DefaultQuality;

            if (string.IsNullOrWhiteSpace(output))
            {
                if (json)
                    CommandOutput.WriteJsonError("invalid_output", "Output path is required. Use --output to specify the MP4 file.");
                else
                    AnsiConsole.MarkupLine("[red]Error:[/] Output path is required. Use --output to specify the MP4 file.");
                return;
            }

            if (duration <= 0)
            {
                if (json)
                    CommandOutput.WriteJsonError("invalid_duration", "Duration must be greater than 0.");
                else
                    AnsiConsole.MarkupLine("[red]Error:[/] Duration must be greater than 0.");
                return;
            }

            if (!json)
                AnsiConsole.MarkupLine($"[cyan]Recording game window for {duration} seconds...[/]");

            RecordResult result = await Task.Run(() => CaptureRecordingAsync(output, duration, width, height, framerate, quality, ct), ct);

            if (json)
            {
                CommandOutput.WriteJson(result);
                return;
            }

            if (result.Success)
            {
                AnsiConsole.MarkupLine($"[green]Recording saved:[/] {Markup.Escape(result.Path ?? string.Empty)}");
                AnsiConsole.MarkupLine($"  Duration: {result.Duration}s, Resolution: {result.Width}x{result.Height}");
                AnsiConsole.MarkupLine($"  Size: {FormatFileSize(result.FileSize)}");
            }
            else
            {
                AnsiConsole.MarkupLine($"[red]Recording failed:[/] {Markup.Escape(result.Error ?? "Unknown error")}");
            }
        });

        return command;
    }

    /// <summary>
    /// Captures a recording using ScreenRecorderLib with WindowRecordingSource.
    /// </summary>
    [SupportedOSPlatform("windows")]
    private static RecordResult CaptureRecordingAsync(string outputPath, int durationSec, int width, int height, int framerate, int quality, CancellationToken ct)
    {
        try
        {
            // Find the game process by window title
            var gameProcess = Process.GetProcesses()
                .FirstOrDefault(p => !string.IsNullOrEmpty(p.MainWindowTitle) && p.MainWindowTitle.Contains(GameWindowTitle));

            if (gameProcess?.MainWindowHandle == null || gameProcess.MainWindowHandle == IntPtr.Zero)
            {
                return new RecordResult
                {
                    Success = false,
                    Error = "Game window not found. Make sure 'Diplomacy is Not an Option' is running."
                };
            }

            IntPtr hwnd = gameProcess.MainWindowHandle;

            // Ensure output directory exists
            string? outputDir = Path.GetDirectoryName(outputPath);
            if (!string.IsNullOrEmpty(outputDir) && !Directory.Exists(outputDir))
            {
                Directory.CreateDirectory(outputDir);
            }

            // Configure recorder options
            var options = new ScreenRecorderLibRec.RecorderOptions
            {
                OutputOptions = new ScreenRecorderLibRec.OutputOptions
                {
                    RecorderMode = ScreenRecorderLibRec.RecorderMode.Video,
                    OutputFrameSize = new ScreenRecorderLibRec.ScreenSize(width, height),
                    Stretch = ScreenRecorderLibRec.StretchMode.Fill,
                    IsVideoFramePreviewEnabled = false,
                    IsVideoCaptureEnabled = true,
                },
                VideoEncoderOptions = new ScreenRecorderLibRec.VideoEncoderOptions
                {
                    Framerate = framerate,
                    Quality = quality,
                    IsFixedFramerate = true,
                    IsHardwareEncodingEnabled = true,
                    IsMp4FastStartEnabled = true,
                    IsFragmentedMp4Enabled = false,
                    IsLowLatencyEnabled = false,
                    IsThrottlingDisabled = false,
                },
                AudioOptions = new ScreenRecorderLibRec.AudioOptions
                {
                    IsAudioEnabled = false,
                    IsInputDeviceEnabled = false,
                    IsOutputDeviceEnabled = false,
                },
                SourceOptions = new ScreenRecorderLibRec.SourceOptions
                {
                    RecordingSources =
                    {
                        new ScreenRecorderLibRec.WindowRecordingSource(hwnd)
                        {
                            IsCursorCaptureEnabled = false,
                            IsVideoCaptureEnabled = true
                        }
                    }
                }
            };

            using var recorder = ScreenRecorderLibRec.Recorder.CreateRecorder(options);
            using var complete = new ManualResetEventSlim(false);
            string? failure = null;

            recorder.OnRecordingFailed += (_, args) =>
            {
                failure = args?.Error ?? "Recording failed.";
                complete.Set();
            };

            recorder.OnRecordingComplete += (_, _) =>
            {
                complete.Set();
            };

            recorder.Record(outputPath);

            // Wait for the specified duration
            if (!complete.Wait(TimeSpan.FromSeconds(durationSec)))
            {
                // Check if cancelled
                if (ct.IsCancellationRequested)
                {
                    recorder.Stop();
                    return new RecordResult
                    {
                        Success = false,
                        Error = "Recording cancelled."
                    };
                }
            }

            recorder.Stop();

            // Wait for completion event with timeout
            if (!complete.Wait(TimeSpan.FromSeconds(Math.Max(10, durationSec + 15))))
            {
                return new RecordResult
                {
                    Success = false,
                    Error = "Recording timed out waiting for completion."
                };
            }

            if (!string.IsNullOrWhiteSpace(failure))
            {
                return new RecordResult
                {
                    Success = false,
                    Error = failure
                };
            }

            if (!File.Exists(outputPath) || new FileInfo(outputPath).Length < 4096)
            {
                return new RecordResult
                {
                    Success = false,
                    Error = "Recording file was not created or was empty."
                };
            }

            return new RecordResult
            {
                Success = true,
                Path = outputPath,
                Duration = durationSec,
                Width = width,
                Height = height,
                FileSize = new FileInfo(outputPath).Length
            };
        }
        catch (OperationCanceledException)
        {
            return new RecordResult
            {
                Success = false,
                Error = "Recording cancelled."
            };
        }
        catch (Exception ex)
        {
            return new RecordResult
            {
                Success = false,
                Error = ex.Message
            };
        }
    }

    private static string FormatFileSize(long bytes)
    {
        string[] sizes = { "B", "KB", "MB", "GB" };
        int order = 0;
        double size = bytes;
        while (size >= 1024 && order < sizes.Length - 1)
        {
            order++;
            size /= 1024;
        }
        return $"{size:0.##} {sizes[order]}";
    }
}

/// <summary>
/// Result of a recording operation.
/// </summary>
internal record RecordResult
{
    public bool Success { get; init; }
    public string? Path { get; init; }
    public string? Error { get; init; }
    public int Duration { get; init; }
    public int Width { get; init; }
    public int Height { get; init; }
    public long FileSize { get; init; }
}
