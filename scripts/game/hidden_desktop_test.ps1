#Requires -Version 5.1
<#
.SYNOPSIS
Validates whether Unity D3D11 games render correctly on Win32 hidden desktops.

.DESCRIPTION
This script is the critical gate for DINOForge agent isolation. It determines whether
the game can run completely invisibly using a hidden desktop, enabling zero-focus-interference
agent automation.

Test sequence:
1. Create hidden desktop "DINOForge_Test"
2. Launch game on that desktop
3. Wait 15s for load
4. Capture screenshot via BitBlt
5. Report: SUCCESS (non-black frame) or FAILURE
6. Cleanup

.EXAMPLE
pwsh -File scripts\game\hidden_desktop_test.ps1
#>

param(
    [string]$GameExePath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
    [string]$GameDir = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option",
    [int]$WaitSeconds = 15,
    [string]$OutputDir = "$env:TEMP\DINOForge",
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"
$VerbosePreference = if ($Verbose) { "Continue" } else { "SilentlyContinue" }

# Ensure output directory exists
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force > $null
    Write-Verbose "Created output directory: $OutputDir"
}

$outputPath = Join-Path $OutputDir "hidden_desktop_test.png"

Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Hidden Desktop Rendering Test for DINOForge" -ForegroundColor Cyan
Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Game: $GameExePath" -ForegroundColor Gray
Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Output: $outputPath" -ForegroundColor Gray

# ============================================================================
# P/Invoke Definitions
# ============================================================================

# Load required assemblies
Add-Type -AssemblyName System.Drawing
# Load required assemblies
Add-Type -AssemblyName System.Drawing

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Drawing;

public class Win32Desktop {
    [DllImport("user32.dll", SetLastError=true)]
    public static extern IntPtr CreateDesktop(
        string lpszDesktop,
        IntPtr lpszDevice,
        IntPtr pDevMode,
        uint dwFlags,
        uint dwDesiredAccess,
        IntPtr lpsa
    );

    [DllImport("user32.dll", SetLastError=true)]
    public static extern bool CloseDesktop(IntPtr hDesktop);

    [DllImport("user32.dll", SetLastError=true)]
    public static extern bool SetThreadDesktop(IntPtr hDesktop);

    [DllImport("user32.dll")]
    public static extern IntPtr GetThreadDesktop(uint dwThreadId);

    [DllImport("kernel32.dll")]
    public static extern uint GetCurrentThreadId();

    // Constants
    public const uint DESKTOP_ALL_ACCESS = 0x01FF;
    public const uint DF_ALLOWOTHERACCOUNTHOOK = 0x0001;
}

public class Win32Process {
    [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)]
    public static extern bool CreateProcess(
        string lpApplicationName,
        string lpCommandLine,
        IntPtr lpProcessAttributes,
        IntPtr lpThreadAttributes,
        bool bInheritHandles,
        uint dwCreationFlags,
        IntPtr lpEnvironment,
        string lpCurrentDirectory,
        ref STARTUPINFO lpStartupInfo,
        out PROCESS_INFORMATION lpProcessInformation
    );

    [DllImport("kernel32.dll", SetLastError=true)]
    public static extern bool TerminateProcess(IntPtr hProcess, uint uExitCode);

    [DllImport("kernel32.dll", SetLastError=true)]
    public static extern bool CloseHandle(IntPtr hObject);

    [DllImport("kernel32.dll")]
    public static extern uint WaitForSingleObject(IntPtr hHandle, uint dwMilliseconds);

    [DllImport("kernel32.dll", SetLastError=true)]
    public static extern bool GetProcessTimes(
        IntPtr hProcess,
        out long lpCreationTime,
        out long lpExitTime,
        out long lpKernelTime,
        out long lpUserTime
    );

    [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)]
    public struct STARTUPINFO {
        public int cb;
        public string lpReserved;
        public string lpDesktop;
        public string lpTitle;
        public int dwX;
        public int dwY;
        public int dwXSize;
        public int dwYSize;
        public int dwXCountChars;
        public int dwYCountChars;
        public int dwFillAttribute;
        public int dwFlags;
        public short wShowWindow;
        public short cbReserved2;
        public IntPtr lpReserved2;
        public IntPtr hStdInput;
        public IntPtr hStdOutput;
        public IntPtr hStdError;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct PROCESS_INFORMATION {
        public IntPtr hProcess;
        public IntPtr hThread;
        public int dwProcessId;
        public int dwThreadId;
    }

    public const uint CREATE_NEW_CONSOLE = 0x00000010;
    public const uint CREATE_NO_WINDOW = 0x08000000;
    public const uint NORMAL_PRIORITY_CLASS = 0x00000020;
    public const uint INFINITE = 0xFFFFFFFF;
    public const uint WAIT_TIMEOUT = 258;
}

public class Win32Window {
    [DllImport("user32.dll", CharSet=CharSet.Auto, SetLastError=true)]
    public static extern IntPtr FindWindow(IntPtr lpClassName, string lpWindowName);

    [DllImport("user32.dll", CharSet=CharSet.Auto, SetLastError=true)]
    public static extern IntPtr FindWindowEx(IntPtr parentHandle, IntPtr childAfter, IntPtr className, string windowTitle);

    [DllImport("user32.dll", SetLastError=true)]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll", SetLastError=true)]
    public static extern bool IsWindow(IntPtr hWnd);

    [DllImport("user32.dll", SetLastError=true)]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

    [DllImport("user32.dll", SetLastError=true)]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    [DllImport("user32.dll")]
    public static extern IntPtr GetDesktopWindow();

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    public const int SW_HIDE = 0;
    public const int SW_SHOW = 5;
}

public class Win32Gdi {
    [DllImport("user32.dll", SetLastError=true)]
    public static extern IntPtr GetDC(IntPtr hWnd);

    [DllImport("user32.dll", SetLastError=true)]
    public static extern int ReleaseDC(IntPtr hWnd, IntPtr hDC);

    [DllImport("gdi32.dll", SetLastError=true)]
    public static extern IntPtr CreateCompatibleDC(IntPtr hdc);

    [DllImport("gdi32.dll", SetLastError=true)]
    public static extern IntPtr CreateCompatibleBitmap(IntPtr hdc, int nWidth, int nHeight);

    [DllImport("gdi32.dll", SetLastError=true)]
    public static extern IntPtr SelectObject(IntPtr hdc, IntPtr hgdiobj);

    [DllImport("gdi32.dll", SetLastError=true)]
    public static extern bool BitBlt(
        IntPtr hdcDest,
        int nXDest,
        int nYDest,
        int nWidth,
        int nHeight,
        IntPtr hdcSrc,
        int nXSrc,
        int nYSrc,
        uint dwRop
    );

    [DllImport("gdi32.dll", SetLastError=true)]
    public static extern bool DeleteDC(IntPtr hdc);

    [DllImport("gdi32.dll", SetLastError=true)]
    public static extern bool DeleteObject(IntPtr hObject);

    [DllImport("user32.dll", CharSet=CharSet.Auto)]
    public static extern int GetWindowTextLength(IntPtr hWnd);

    public const uint SRCCOPY = 0x00CC0020;
}
"@ -Language CSharp -ErrorAction Stop

Write-Verbose "P/Invoke definitions loaded"

# ============================================================================
# Helper Functions
# ============================================================================

function Test-GameExists {
    if (-not (Test-Path $GameExePath)) {
        Write-Error "Game executable not found: $GameExePath"
        exit 1
    }
    Write-Verbose "Game exists: $GameExePath"
}

function Create-HiddenDesktop {
    param([string]$DesktopName)

    Write-Verbose "Creating hidden desktop: $DesktopName"

    $handle = [Win32Desktop]::CreateDesktop(
        $DesktopName,
        [IntPtr]::Zero,
        [IntPtr]::Zero,
        0,
        [Win32Desktop]::DESKTOP_ALL_ACCESS,
        [IntPtr]::Zero
    )

    if ($handle -eq [IntPtr]::Zero) {
        $err = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
        Write-Error "Failed to create desktop: error code $err"
        exit 1
    }

    Write-Host "✓ Created hidden desktop: $DesktopName" -ForegroundColor Green
    return $handle
}

function Launch-GameOnDesktop {
    param([string]$DesktopName)

    Write-Verbose "Launching game on desktop: $DesktopName"

    $startupInfo = New-Object Win32Process+STARTUPINFO
    $startupInfo.cb = [System.Runtime.InteropServices.Marshal]::SizeOf($startupInfo)
    $startupInfo.lpDesktop = "WinSta0\$DesktopName"
    $startupInfo.dwFlags = 0x00000100  # STARTF_USESTDHANDLES
    $startupInfo.hStdInput = [IntPtr]::Zero
    $startupInfo.hStdOutput = [IntPtr]::Zero
    $startupInfo.hStdError = [IntPtr]::Zero

    $processInfo = New-Object Win32Process+PROCESS_INFORMATION

    $success = [Win32Process]::CreateProcess(
        $GameExePath,
        '""',
        [IntPtr]::Zero,
        [IntPtr]::Zero,
        $false,
        [Win32Process]::CREATE_NO_WINDOW -bor [Win32Process]::NORMAL_PRIORITY_CLASS,
        [IntPtr]::Zero,
        $GameDir,
        [ref]$startupInfo,
        [ref]$processInfo
    )

    if (-not $success) {
        $err = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
        Write-Error "Failed to launch game: error code $err"
        exit 1
    }

    Write-Host "✓ Launched game on hidden desktop (PID: $($processInfo.dwProcessId))" -ForegroundColor Green

    [Win32Process]::CloseHandle($processInfo.hThread)

    return @{
        ProcessHandle = $processInfo.hProcess
        ProcessId = $processInfo.dwProcessId
    }
}

function Wait-ForGameWindow {
    param([int]$TimeoutSeconds = 15)

    Write-Verbose "Waiting for game window (timeout: ${TimeoutSeconds}s)"

    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

    while ($stopwatch.Elapsed.TotalSeconds -lt $TimeoutSeconds) {
        # Look for any window with "Diplomacy" in the title
        $windowHandle = [Win32Window]::FindWindow([IntPtr]::Zero, "Diplomacy is Not an Option")

        if ($windowHandle -ne [IntPtr]::Zero -and [Win32Window]::IsWindow($windowHandle)) {
            $stopwatch.Stop()
            Write-Host "✓ Game window found after $([Math]::Round($stopwatch.Elapsed.TotalSeconds, 1))s" -ForegroundColor Green
            return $windowHandle
        }

        Start-Sleep -Milliseconds 500
    }

    Write-Error "Game window not found after ${TimeoutSeconds}s"
    return [IntPtr]::Zero
}

function Capture-WindowScreenshot {
    param(
        [IntPtr]$WindowHandle,
        [string]$OutputPath
    )

    Write-Verbose "Capturing screenshot from window handle: $WindowHandle"

    $rect = New-Object Win32Window+RECT
    if (-not [Win32Window]::GetWindowRect($WindowHandle, [ref]$rect)) {
        Write-Error "Failed to get window rect"
        return $false
    }

    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top

    Write-Verbose "Window dimensions: ${width}x${height}"

    if ($width -le 0 -or $height -le 0) {
        Write-Error "Invalid window dimensions: ${width}x${height}"
        return $false
    }

    # Get device context from window
    $srcDC = [Win32Gdi]::GetDC($WindowHandle)
    if ($srcDC -eq [IntPtr]::Zero) {
        Write-Error "Failed to get window DC"
        return $false
    }

    # Create compatible DC and bitmap
    $dstDC = [Win32Gdi]::CreateCompatibleDC($srcDC)
    if ($dstDC -eq [IntPtr]::Zero) {
        [Win32Gdi]::ReleaseDC($WindowHandle, $srcDC)
        Write-Error "Failed to create compatible DC"
        return $false
    }

    $hBmp = [Win32Gdi]::CreateCompatibleBitmap($srcDC, $width, $height)
    if ($hBmp -eq [IntPtr]::Zero) {
        [Win32Gdi]::DeleteDC($dstDC)
        [Win32Gdi]::ReleaseDC($WindowHandle, $srcDC)
        Write-Error "Failed to create compatible bitmap"
        return $false
    }

    # Select bitmap into DC and perform BitBlt
    $oldBmp = [Win32Gdi]::SelectObject($dstDC, $hBmp)
    if (-not [Win32Gdi]::BitBlt($dstDC, 0, 0, $width, $height, $srcDC, 0, 0, [Win32Gdi]::SRCCOPY)) {
        $err = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
        Write-Error "BitBlt failed: error code $err"
        [Win32Gdi]::SelectObject($dstDC, $oldBmp)
        [Win32Gdi]::DeleteObject($hBmp)
        [Win32Gdi]::DeleteDC($dstDC)
        [Win32Gdi]::ReleaseDC($WindowHandle, $srcDC)
        return $false
    }

    # Convert HBITMAP to managed Bitmap and save
    try {
        $bmp = [System.Drawing.Bitmap]::FromHbitmap($hBmp)
        $bmp.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
        Write-Host "✓ Screenshot saved: $OutputPath ($($bmp.Width)x$($bmp.Height))" -ForegroundColor Green
        $bmp.Dispose()
        $result = $true
    } catch {
        Write-Error "Failed to save bitmap: $_"
        $result = $false
    }

    # Cleanup
    [Win32Gdi]::SelectObject($dstDC, $oldBmp)
    [Win32Gdi]::DeleteObject($hBmp)
    [Win32Gdi]::DeleteDC($dstDC)
    [Win32Gdi]::ReleaseDC($WindowHandle, $srcDC)

    return $result
}

function Test-ScreenshotNotBlack {
    param([string]$ImagePath)

    Write-Verbose "Analyzing screenshot: $ImagePath"

    if (-not (Test-Path $ImagePath)) {
        Write-Error "Screenshot not found: $ImagePath"
        return $false
    }

    try {
        $bmp = [System.Drawing.Bitmap]::new($ImagePath)

        # Sample pixels at various locations (not just the corners)
        $width = $bmp.Width
        $height = $bmp.Height

        $samples = @(
            @{ x = [int]($width * 0.25); y = [int]($height * 0.25) },
            @{ x = [int]($width * 0.5);  y = [int]($height * 0.5)  },
            @{ x = [int]($width * 0.75); y = [int]($height * 0.75) },
            @{ x = [int]($width * 0.1);  y = [int]($height * 0.1)  },
            @{ x = [int]($width * 0.9);  y = [int]($height * 0.9)  }
        )

        $nonBlackPixels = 0

        foreach ($sample in $samples) {
            $pixel = $bmp.GetPixel($sample.x, $sample.y)

            # Check if pixel is significantly different from black (0,0,0)
            # Allow for very dark grays (e.g., shadow areas)
            if ($pixel.R -gt 10 -or $pixel.G -gt 10 -or $pixel.B -gt 10) {
                $nonBlackPixels++
            }
        }

        $bmp.Dispose()

        if ($nonBlackPixels -ge 3) {
            Write-Verbose "Non-black pixels detected: $nonBlackPixels / 5 samples"
            return $true
        } else {
            Write-Verbose "Screenshot appears to be mostly black"
            return $false
        }
    } catch {
        Write-Error "Failed to analyze screenshot: $_"
        return $false
    }
}

function Close-HiddenDesktop {
    param([IntPtr]$Handle)

    Write-Verbose "Closing desktop handle: $Handle"
    [Win32Desktop]::CloseDesktop($Handle) | Out-Null
    Write-Verbose "Desktop closed"
}

function Stop-GameProcess {
    param([IntPtr]$ProcessHandle, [int]$ProcessId)

    Write-Verbose "Terminating game process: PID=$ProcessId"

    # Try graceful terminate first
    [Win32Process]::TerminateProcess($ProcessHandle, 0) | Out-Null

    # Wait a bit for process to die
    Start-Sleep -Milliseconds 500

    # Also try via Get-Process as backup
    $proc = Get-Process -Id $ProcessId -ErrorAction SilentlyContinue
    if ($proc) {
        Stop-Process -Id $ProcessId -Force -ErrorAction SilentlyContinue
        Write-Verbose "Process terminated via Stop-Process"
    } else {
        Write-Verbose "Process already terminated"
    }

    [Win32Process]::CloseHandle($ProcessHandle) | Out-Null
}

# ============================================================================
# Main Test Execution
# ============================================================================

$testResult = $null
$desktopHandle = [IntPtr]::Zero
$gameProcess = $null

try {
    # Verify game exists
    Test-GameExists

    # Create hidden desktop
    $desktopName = "DINOForge_Test_$(Get-Random -Minimum 1000 -Maximum 9999)"
    $desktopHandle = Create-HiddenDesktop -DesktopName $desktopName

    # Launch game on hidden desktop
    $gameProcess = Launch-GameOnDesktop -DesktopName $desktopName

    # Wait for game window to appear
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Waiting for game to load..." -ForegroundColor Gray
    Start-Sleep -Seconds 2  # Initial grace period

    $windowHandle = Wait-ForGameWindow -TimeoutSeconds $WaitSeconds

    if ($windowHandle -eq [IntPtr]::Zero) {
        Write-Host "✗ FAILURE: Game window not found within ${WaitSeconds}s" -ForegroundColor Red
        $testResult = $false
    } else {
        # Wait a bit more for rendering to start
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Game window found, waiting for rendering..." -ForegroundColor Gray
        Start-Sleep -Seconds 3

        # Capture screenshot
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Capturing screenshot..." -ForegroundColor Gray
        $captureSuccess = Capture-WindowScreenshot -WindowHandle $windowHandle -OutputPath $outputPath

        if ($captureSuccess) {
            # Analyze screenshot
            $isNotBlack = Test-ScreenshotNotBlack -ImagePath $outputPath

            if ($isNotBlack) {
                Write-Host "✓ SUCCESS: Game rendered non-black frame on hidden desktop!" -ForegroundColor Green
                Write-Host "  This means agents can run the game invisibly with zero focus interference." -ForegroundColor Green
                $testResult = $true
            } else {
                Write-Host "✗ FAILURE: Screenshot is mostly black (no rendering detected)" -ForegroundColor Red
                Write-Host "  Fallback: Use separate Windows user account for agent isolation." -ForegroundColor Yellow
                $testResult = $false
            }
        } else {
            Write-Host "✗ FAILURE: Could not capture screenshot from hidden desktop window" -ForegroundColor Red
            $testResult = $false
        }
    }

} catch {
    Write-Host "✗ FAILURE: Unexpected error: $_" -ForegroundColor Red
    $testResult = $false
} finally {
    # Cleanup
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Cleaning up..." -ForegroundColor Gray

    if ($gameProcess) {
        Stop-GameProcess -ProcessHandle $gameProcess.ProcessHandle -ProcessId $gameProcess.ProcessId
        Write-Verbose "Game process terminated"
    }

    if ($desktopHandle -ne [IntPtr]::Zero) {
        Close-HiddenDesktop -Handle $desktopHandle
        Write-Verbose "Desktop closed"
    }
}

# ============================================================================
# Final Report
# ============================================================================

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "TEST RESULT" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

if ($testResult -eq $true) {
    Write-Host ""
    Write-Host "Status: ✓ SUCCESS" -ForegroundColor Green
    Write-Host ""
    Write-Host "Finding:" -ForegroundColor Green
    Write-Host "  Unity D3D11 rendering WORKS on hidden desktops"
    Write-Host ""
    Write-Host "Implications:" -ForegroundColor Green
    Write-Host "  • Agents can launch game completely invisibly"
    Write-Host "  • Zero window focus interference with user's session"
    Write-Host "  • Safe for concurrent automation + user work"
    Write-Host "  • Implementation: Use CreateDesktop API in game launcher"
    Write-Host ""
    Write-Host "Next Steps:" -ForegroundColor Green
    Write-Host "  1. Update GameLauncher in MCP server to use hidden desktops by default"
    Write-Host "  2. Add desktop management to Runtime bridge (create/destroy/list)"
    Write-Host "  3. Document in README: 'Agent Mode' (hidden desktop rendering)"
    Write-Host "  4. Create slash command: /launch-game-agent (uses hidden desktop)"
    Write-Host ""
    exit 0

} elseif ($testResult -eq $false) {
    Write-Host ""
    Write-Host "Status: ✗ FAILURE" -ForegroundColor Red
    Write-Host ""
    Write-Host "Finding:" -ForegroundColor Red
    Write-Host "  Unity D3D11 rendering DOES NOT work on hidden desktops"
    Write-Host "  (or game failed to launch entirely)"
    Write-Host ""
    Write-Host "Implications:" -ForegroundColor Yellow
    Write-Host "  • Hidden desktop approach is NOT viable for agent isolation"
    Write-Host "  • Must fall back to separate Windows user account"
    Write-Host "  • Each game instance requires dedicated user session"
    Write-Host ""
    Write-Host "Next Steps:" -ForegroundColor Yellow
    Write-Host "  1. Implement user account isolation (via runas.exe)"
    Write-Host "  2. Set up test game directory at known path per user"
    Write-Host "  3. Update launcher to use separate credential token"
    Write-Host "  4. Document in README: 'Isolated Mode' (separate user account)"
    Write-Host "  5. Create slash command: /launch-game-isolated (uses separate user)"
    Write-Host ""
    Write-Host "Screenshot for debugging: $outputPath" -ForegroundColor Yellow
    Write-Host ""
    exit 1

} else {
    Write-Host ""
    Write-Host "Status: ? UNKNOWN" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Investigation needed - review logs above" -ForegroundColor Yellow
    Write-Host ""
    exit 2
}
