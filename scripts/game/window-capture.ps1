#Requires -Version 5.1
<#
.SYNOPSIS
    Window-isolated screenshot and clip capture using ScreenRecorderLib / Windows.Graphics.Capture.

.DESCRIPTION
    Records or snapshots a specific game window by process ID or window-title match.
    This avoids desktop cropping and does not depend on ffmpeg gdigrab.

.NOTES
    The helper searches for ScreenRecorderLib.dll in the local tools build output first,
    then falls back to the NuGet cache.
#>

param(
    [ValidateSet("snapshot", "record")]
    [string]$Mode,

    [string]$WindowTitle,
    [int]$ProcessId = 0,
    [string]$OutputPath,
    [int]$DurationSec = 6,
    [int]$Width = 1280,
    [int]$Height = 800,
    [int]$Framerate = 30,
    [int]$Quality = 80,
    [int]$WindowTimeoutSec = 30
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Resolve-ScreenRecorderLibDll {
    $repoRoot = Join-Path $PSScriptRoot "..\.."
    $repoCandidates = @(
        (Join-Path $repoRoot "src\Tools\McpServer\bin\x64\Release\net11.0\ScreenRecorderLib.dll"),
        (Join-Path $repoRoot "src\Tools\McpServer\bin\x64\Debug\net11.0\ScreenRecorderLib.dll")
    )

    foreach ($candidate in $repoCandidates) {
        if (Test-Path $candidate) {
            return (Resolve-Path $candidate).Path
        }
    }

    $nugetCandidates = @(
        Join-Path $env:USERPROFILE ".nuget\packages\screenrecorderlib\6.6.0\build\x64\ScreenRecorderLib.dll",
        Join-Path $env:USERPROFILE ".nuget\packages\screenrecorderlib\6.0.0\build\x64\ScreenRecorderLib.dll"
    )

    foreach ($candidate in $nugetCandidates) {
        if (Test-Path $candidate) {
            return (Resolve-Path $candidate).Path
        }
    }

    throw "ScreenRecorderLib.dll was not found in the tools build output or NuGet cache."
}

function Initialize-WindowCaptureTypes {
    $dll = Resolve-ScreenRecorderLibDll
    [void][System.Reflection.Assembly]::LoadFrom($dll)

    Add-Type -ReferencedAssemblies $dll -TypeDefinition @"
using System;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using ScreenRecorderLib;

public static class DinoWindowCapture
{
    private static RecorderOptions CreateOptions(IntPtr hwnd, int width, int height, int framerate, int quality)
    {
        RecorderOptions options = RecorderOptions.Default;

        options.OutputOptions = new OutputOptions
        {
            RecorderMode = RecorderMode.Video,
            OutputFrameSize = new ScreenSize(width, height),
            Stretch = StretchMode.Fill,
            IsVideoFramePreviewEnabled = false,
            IsVideoCaptureEnabled = true,
        };

        options.VideoEncoderOptions = new VideoEncoderOptions
        {
            Framerate = framerate,
            Quality = quality,
            IsFixedFramerate = true,
            IsHardwareEncodingEnabled = true,
            IsMp4FastStartEnabled = true,
            IsFragmentedMp4Enabled = false,
            IsLowLatencyEnabled = false,
            IsThrottlingDisabled = false,
        };

        options.AudioOptions = new AudioOptions
        {
            IsAudioEnabled = false,
            IsInputDeviceEnabled = false,
            IsOutputDeviceEnabled = false,
        };

        options.SourceOptions = new SourceOptions
        {
            RecordingSources = new List<RecordingSourceBase>
            {
                new WindowRecordingSource(hwnd)
                {
                    IsCursorCaptureEnabled = false,
                    IsVideoCaptureEnabled = true
                }
            }
        };

        return options;
    }

    public static void TakeSnapshot(IntPtr hwnd, string outputPath, int width, int height, int framerate, int quality)
    {
        if (hwnd == IntPtr.Zero)
        {
            throw new InvalidOperationException("Invalid window handle.");
        }

        RecorderOptions options = CreateOptions(hwnd, width, height, framerate, quality);
        using Recorder recorder = Recorder.CreateRecorder(options);
        bool ok = recorder.TakeSnapshot(outputPath);
        if (!ok)
        {
            throw new InvalidOperationException("ScreenRecorderLib.TakeSnapshot returned false.");
        }

        if (!File.Exists(outputPath) || new FileInfo(outputPath).Length < 1024)
        {
            throw new InvalidOperationException("Snapshot file was not created or was empty.");
        }
    }

    public static void RecordClip(IntPtr hwnd, string outputPath, int durationSec, int width, int height, int framerate, int quality)
    {
        if (hwnd == IntPtr.Zero)
        {
            throw new InvalidOperationException("Invalid window handle.");
        }

        if (durationSec <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(durationSec));
        }

        RecorderOptions options = CreateOptions(hwnd, width, height, framerate, quality);
        using Recorder recorder = Recorder.CreateRecorder(options);

        using ManualResetEventSlim complete = new ManualResetEventSlim(false);
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
        Thread.Sleep(TimeSpan.FromSeconds(durationSec));
        recorder.Stop();

        if (!complete.Wait(TimeSpan.FromSeconds(Math.Max(10, durationSec + 15))))
        {
            throw new TimeoutException("Timed out waiting for recording completion.");
        }

        if (!string.IsNullOrWhiteSpace(failure))
        {
            throw new InvalidOperationException(failure);
        }

        if (!File.Exists(outputPath) || new FileInfo(outputPath).Length < 4096)
        {
            throw new InvalidOperationException("Recording file was not created or was empty.");
        }
    }
}
"@
}

function Resolve-TargetWindowHandle {
    param(
        [string]$TargetWindowTitle,
        [int]$TargetProcessId,
        [int]$TimeoutSec
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSec)
    while ((Get-Date) -lt $deadline) {
        try {
            if ($TargetProcessId -gt 0) {
                $proc = Get-Process -Id $TargetProcessId -ErrorAction SilentlyContinue
                if ($proc -and $proc.MainWindowHandle -ne 0) {
                    return [IntPtr]$proc.MainWindowHandle
                }
            }
            elseif (-not [string]::IsNullOrWhiteSpace($TargetWindowTitle)) {
                $proc = Get-Process | Where-Object {
                    $_.MainWindowTitle -and $_.MainWindowTitle -like "*$TargetWindowTitle*"
                } | Select-Object -First 1
                if ($proc -and $proc.MainWindowHandle -ne 0) {
                    return [IntPtr]$proc.MainWindowHandle
                }
            }
        }
        catch {
            # Keep polling until timeout.
        }

        Start-Sleep -Milliseconds 250
    }

    throw "Timed out waiting for the target window handle."
}

function Invoke-WindowCapture {
    param(
        [ValidateSet("snapshot", "record")]
        [string]$CaptureMode,
        [string]$CaptureWindowTitle,
        [int]$CaptureProcessId,
        [string]$CaptureOutputPath,
        [int]$CaptureDurationSec,
        [int]$CaptureWidth,
        [int]$CaptureHeight,
        [int]$CaptureFramerate,
        [int]$CaptureQuality,
        [int]$CaptureWindowTimeoutSec
    )

    if ([string]::IsNullOrWhiteSpace($CaptureOutputPath)) {
        throw "OutputPath is required."
    }

    Initialize-WindowCaptureTypes
    $hwnd = Resolve-TargetWindowHandle -TargetWindowTitle $CaptureWindowTitle -TargetProcessId $CaptureProcessId -TimeoutSec $CaptureWindowTimeoutSec

    $outputDir = Split-Path -Parent $CaptureOutputPath
    if ($outputDir -and -not (Test-Path $outputDir)) {
        New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
    }

    if ($CaptureMode -eq "snapshot") {
        [DinoWindowCapture]::TakeSnapshot($hwnd, $CaptureOutputPath, $CaptureWidth, $CaptureHeight, $CaptureFramerate, $CaptureQuality)
        return
    }

    [DinoWindowCapture]::RecordClip($hwnd, $CaptureOutputPath, $CaptureDurationSec, $CaptureWidth, $CaptureHeight, $CaptureFramerate, $CaptureQuality)
}

Invoke-WindowCapture -CaptureMode $Mode -CaptureWindowTitle $WindowTitle -CaptureProcessId $ProcessId -CaptureOutputPath $OutputPath -CaptureDurationSec $DurationSec -CaptureWidth $Width -CaptureHeight $Height -CaptureFramerate $Framerate -CaptureQuality $Quality -CaptureWindowTimeoutSec $WindowTimeoutSec
