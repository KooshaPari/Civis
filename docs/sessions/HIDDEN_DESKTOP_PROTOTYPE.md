# Hidden Desktop Prototype for DINOForge Game Automation

**Status:** Research + Prototype (Complete)
**Date:** 2026-03-25
**Purpose:** Non-intrusive game automation using Win32 hidden desktop isolation
**Target:** Run DINO game instances on invisible desktops while user works normally

---

## Executive Summary

A **Win32 hidden desktop** approach provides OS-level process isolation without VM overhead. A game running on a hidden desktop:
- Cannot steal focus from the user's visible desktop
- Remains invisible (no window flickering, no visual interference)
- Can still accept input via Win32 APIs (PostMessage, SendInput)
- Can be screenshotted via GDI BitBlt (cross-desktop capable, unlike WGC)
- Runs at full performance (same machine, no network MTU/latency)
- Requires NO modification to game, BepInEx, or DINOForge code

**Key Finding:** This is viable and more efficient than VM/container approaches, with one critical caveat: **Unity rendering on backgrounded desktops is untested** (risk category: Medium). Prototype testing required before production use.

---

## Win32 Desktop Isolation Mechanism

### Desktop API Overview

Windows provides a four-level object hierarchy:

```
WindowStation (namespace for desktops)
  ├─ Desktop_1 (user's visible desktop, default)
  ├─ Desktop_2 (hidden, agent uses this)
  ├─ Desktop_N (other isolated desktops)
World (logical output surface)
  ├─ Screen device (DC)
  └─ Window (HWND)
```

**Key APIs:**

| API | Purpose | Signature |
|-----|---------|-----------|
| `CreateDesktop` | Create new hidden desktop | `IntPtr CreateDesktop(string name, string device, IntPtr pDevmode, uint dwFlags, uint dwDesiredAccess, IntPtr psa)` |
| `SetThreadDesktop` | Bind thread to desktop | `bool SetThreadDesktop(IntPtr hDesktop)` |
| `CloseDesktop` | Destroy desktop | `bool CloseDesktop(IntPtr hDesktop)` |
| `OpenDesktop` | Access existing desktop | `IntPtr OpenDesktop(string name, uint dwFlags, bool bInherit, uint dwDesiredAccess)` |

**Process Launch with Desktop Binding:**

Instead of `SetThreadDesktop` (which only works on threadlocal threads), use `CreateProcessW` with `STARTUPINFO.lpDesktop`:

```csharp
STARTUPINFO si = new STARTUPINFO {
    cb = (uint)Marshal.SizeOf(typeof(STARTUPINFO)),
    lpDesktop = "WinSta0\\DINOForge_Agent",  // Backslash-qualified path
    dwFlags = STARTF_USESHOWWINDOW,          // 0x00000001
    wShowWindow = SW_HIDE                    // 0
};

bool success = CreateProcessW(
    lpApplicationName: "C:\\...\\Diplomacy is Not an Option.exe",
    lpCommandLine: null,
    lpProcessAttributes: null,
    lpThreadAttributes: null,
    bInheritHandles: false,
    dwCreationFlags: CREATE_DEFAULT_ERROR_MODE | CREATE_BREAKAWAY_FROM_JOB,
    lpEnvironment: null,
    lpCurrentDirectory: "G:\\SteamLibrary\\steamapps\\common\\Diplomacy is Not an Option",
    lpStartupInfo: ref si,
    lpProcessInformation: out processInfo
);
```

### Why This Works

1. **No Focus Stealing:** Desktops have independent focus chains. Creating a new desktop doesn't affect the user's input focus.
2. **Invisible to User:** The hidden desktop is literally not rendered to the screen—user has zero visual distraction.
3. **Process Independence:** The process running on the hidden desktop is fully autonomous (separate event loops, window messages, input handlers).
4. **Cross-Desktop Input:** Win32 input events (PostMessage, SendInput) work across desktop boundaries if the target window handle is known and the input is routed correctly.

---

## Screenshot Capture Strategy

### Why WGC (Windows.Graphics.Capture) Fails

Windows.Graphics.Capture is **desktop-aware and refuses to capture from a different desktop**. This is a security/design limitation:

```csharp
// This WILL FAIL silently if hWnd is on a different desktop:
var item = await CaptureUIAsync.GetItemsAsync();  // Empty result
```

### Alternative: GDI BitBlt (Cross-Desktop Capable)

GDI `BitBlt` is a lower-level API that can read pixel data from any window's DC, regardless of desktop:

```csharp
[DllImport("user32.dll", SetLastError = true)]
private static extern IntPtr GetDC(IntPtr hWnd);

[DllImport("gdi32.dll", SetLastError = true)]
private static extern bool BitBlt(
    IntPtr hdc,      // Destination DC (screen)
    int x, int y,    // Destination coordinates
    int cx, int cy,  // Width, height
    IntPtr hdcSrc,   // Source DC (hidden desktop window)
    int xSrc, int ySrc,  // Source coordinates
    uint rop         // Raster operation (SRCCOPY = 0x00CC0020)
);

[DllImport("gdi32.dll", SetLastError = true)]
private static extern bool CreateCompatibleBitmap(IntPtr hdc, int cx, int cy);

// Usage:
IntPtr screenDC = GetDC(IntPtr.Zero);  // Gets DC for primary display
IntPtr windowDC = GetDC(hiddenWindowHandle);

// Create bitmap for capture
IntPtr memDC = CreateCompatibleDC(screenDC);
IntPtr bitmap = CreateCompatibleBitmap(screenDC, width, height);
SelectObject(memDC, bitmap);

// Copy pixels from hidden window to memory bitmap
BitBlt(memDC, 0, 0, width, height, windowDC, 0, 0, SRCCOPY);

// Convert bitmap to PNG/JPEG (use GDI+ or Direct2D)
```

### Alternative: DXGI (DirectX, Lower Overhead)

For high-performance repeated captures, DXGI provides GPU-accelerated screen capture:

```csharp
var factory = new Factory1();
var adapter = factory.GetAdapter(0);
var device = new Device(adapter);
var output = adapter.GetOutput(0);

// Create duplicate output (works cross-desktop in some Windows versions)
using (var duplication = output.DuplicateOutput(device))
{
    // Poll for frame updates and capture without stalling the game
}
```

**Note:** DXGI has varying cross-desktop support depending on Windows version (10/11) and driver. GDI BitBlt is more reliable but slower.

### Recommended Approach

For DINOForge MCP game automation:
1. **Primary:** GDI BitBlt (reliable, works cross-desktop, no additional dependencies)
2. **Fallback:** DXGI if performance bottleneck detected
3. **Do NOT use:** WGC (Windows.Graphics.Capture)

---

## Input Delivery to Hidden Desktop

### PostMessage (Safest)

Send keyboard/mouse events directly to the window on the hidden desktop without requiring focus:

```csharp
[DllImport("user32.dll")]
private static extern bool PostMessage(
    IntPtr hWnd,      // Window on hidden desktop
    uint msg,         // WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, etc.
    IntPtr wParam,    // Key code / button flags
    IntPtr lParam     // Scan code, repeat count
);

// Example: Send keypress (Escape key)
const uint WM_KEYDOWN = 0x0100;
const uint WM_KEYUP = 0x0101;
PostMessage(hiddenWindowHandle, WM_KEYDOWN, new IntPtr(0x1B), new IntPtr(0x00010001));  // VK_ESCAPE
PostMessage(hiddenWindowHandle, WM_KEYUP, new IntPtr(0x1B), new IntPtr(0xC0010001));
```

**Advantages:**
- No focus requirements
- Reliable across desktops
- Messages queue properly on target window
- No race conditions with input focus

### SendInput (Requires Focus)

If PostMessage proves insufficient, SendInput can inject events, but only if the process has focus:

```csharp
[DllImport("user32.dll")]
private static extern uint SendInput(uint nInputs, INPUT[] pInputs, int cbSize);

struct INPUT
{
    public InputType type;
    public MOUSEINPUT mi;  // or KEYBDINPUT / HARDWAREINPUT
}

// Usage requires window focus first (via SetForegroundWindow)
// Risk: May briefly steal user's focus
```

**Not recommended for hidden desktop** (might pop window to visible desktop or cause focus flicker).

### MCP Game Input Tool Enhancement

Update `GameInputTool` in `src/Tools/McpServer/Tools/GameInputTool.cs`:

```csharp
public class GameInputTool
{
    private IntPtr _hiddenDesktop;
    private IntPtr _hiddenWindowHandle;

    /// <summary>
    /// Send input to game process, respecting hidden desktop isolation.
    /// Uses PostMessage (cross-desktop safe) instead of SendInput.
    /// </summary>
    public async Task<bool> SendInputAsync(GameInputRequest input)
    {
        // If process is on hidden desktop, use PostMessage
        if (IsProcessOnHiddenDesktop(_gameProcess))
        {
            return SendInputViaPostMessage(_hiddenWindowHandle, input);
        }

        // Otherwise, use standard SendInput (user's visible desktop)
        return SendInputViaWin32(input);
    }

    private bool SendInputViaPostMessage(IntPtr hwnd, GameInputRequest input)
    {
        return input.Type switch
        {
            "keyboard" => SendKeyboardMessageToWindow(hwnd, input.Key, input.IsDown),
            "mouse" => SendMouseMessageToWindow(hwnd, input.X, input.Y, input.Button),
            _ => false
        };
    }
}
```

---

## Prototype: PowerShell Script

### `hidden_desktop_launch.ps1`

**Location:** `scripts/game/hidden_desktop_launch.ps1`
**Purpose:** Launch DINO on hidden desktop + poll for readiness

```powershell
<#
.SYNOPSIS
    Launch DINO game on a hidden Win32 desktop for non-intrusive automation.

.DESCRIPTION
    Creates a new hidden Win32 desktop named "DINOForge_Agent" and launches
    the game process on that desktop. The game runs invisibly to the user,
    allowing uninterrupted automation.

    Captures:
    - Desktop handle (stored in process userdata)
    - Game window handle (for PostMessage input)
    - PID for process monitoring

.PARAMETER GamePath
    Path to DINO game executable.
    Default: G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe

.PARAMETER HiddenDesktopName
    Name of the hidden desktop.
    Default: DINOForge_Agent

.PARAMETER WaitForReady
    Wait up to N seconds for game window to appear.
    Default: 30

.EXAMPLE
    .\hidden_desktop_launch.ps1
    .\hidden_desktop_launch.ps1 -GamePath "D:\Games\DINO\Diplomacy is Not an Option.exe" -WaitForReady 60

#>

[CmdletBinding()]
param(
    [string]$GamePath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
    [string]$HiddenDesktopName = "DINOForge_Agent",
    [int]$WaitForReady = 30
)

$ErrorActionPreference = "Stop"

# ============================================================================
# P/Invoke Declarations
# ============================================================================

$sig = @'
[DllImport("user32.dll", SetLastError = true)]
public static extern IntPtr CreateDesktop(
    string lpszDesktop,
    string lpszDevice,
    IntPtr pDevmode,
    uint dwFlags,
    uint dwDesiredAccess,
    IntPtr psa
);

[DllImport("user32.dll", SetLastError = true)]
public static extern bool CloseDesktop(IntPtr hDesktop);

[DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Auto)]
public static extern bool CreateProcessW(
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

[DllImport("kernel32.dll", SetLastError = true)]
public static extern bool CloseHandle(IntPtr hObject);

[DllImport("user32.dll", SetLastError = true)]
public static extern IntPtr FindWindowEx(
    IntPtr hwndParent,
    IntPtr hwndChildAfter,
    string lpszClass,
    string lpszWindow
);

[DllImport("user32.dll", SetLastError = true)]
public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

[DllImport("kernel32.dll", SetLastError = true)]
public static extern bool GetProcessTimes(
    IntPtr hProcess,
    out System.Runtime.InteropServices.ComTypes.FILETIME lpCreationTime,
    out System.Runtime.InteropServices.ComTypes.FILETIME lpExitTime,
    out System.Runtime.InteropServices.ComTypes.FILETIME lpKernelTime,
    out System.Runtime.InteropServices.ComTypes.FILETIME lpUserTime
);

[StructLayout(LayoutKind.Sequential)]
public struct STARTUPINFO
{
    public uint cb;
    public string lpReserved;
    public string lpDesktop;
    public string lpTitle;
    public uint dwX;
    public uint dwY;
    public uint dwXSize;
    public uint dwYSize;
    public uint dwXCountChars;
    public uint dwYCountChars;
    public uint dwFillAttribute;
    public uint dwFlags;
    public ushort wShowWindow;
    public ushort cbReserved2;
    public IntPtr lpReserved2;
    public IntPtr hStdInput;
    public IntPtr hStdOutput;
    public IntPtr hStdError;
}

[StructLayout(LayoutKind.Sequential)]
public struct PROCESS_INFORMATION
{
    public IntPtr hProcess;
    public IntPtr hThread;
    public uint dwProcessId;
    public uint dwThreadId;
}
'@

Add-Type -MemberDefinition $sig -Name "Win32" -Namespace "PInvoke" -Using System.Text

# ============================================================================
# Constants
# ============================================================================

$STARTF_USESHOWWINDOW = 0x00000001
$SW_HIDE = 0
$SW_SHOW = 5

$DESKTOP_CREATEWINDOW = 0x0002
$DESKTOP_ENUMERATE = 0x0040
$DESKTOP_HOOKCONTROL = 0x0008
$DESKTOP_JOURNAL = 0x0020
$DESKTOP_JOURNALRECORD = 0x0010
$DESKTOP_READOBJECTS = 0x0001
$DESKTOP_SWITCHDESKTOP = 0x0100
$DESKTOP_WRITEOBJECTS = 0x0020

$DESKTOP_ALL_ACCESS = 0x01FF

$DF_ALLOWOTHERACCOUNTHOOK = 0x0001

$CREATE_DEFAULT_ERROR_MODE = 0x04000000
$CREATE_BREAKAWAY_FROM_JOB = 0x01000000

# ============================================================================
# Helper Functions
# ============================================================================

function Write-Status {
    param([string]$Message)
    Write-Host "[*] " -ForegroundColor Cyan -NoNewline
    Write-Host $Message
}

function Write-Success {
    param([string]$Message)
    Write-Host "[+] " -ForegroundColor Green -NoNewline
    Write-Host $Message
}

function Write-Error {
    param([string]$Message)
    Write-Host "[-] " -ForegroundColor Red -NoNewline
    Write-Host $Message
}

function Find-WindowByProcessId {
    param([uint]$PID)

    $hWnd = [PInvoke.Win32]::FindWindowEx([IntPtr]::Zero, [IntPtr]::Zero, $null, $null)

    while ($hWnd -ne [IntPtr]::Zero) {
        $procId = 0
        [PInvoke.Win32]::GetWindowThreadProcessId($hWnd, [ref]$procId) | Out-Null

        if ($procId -eq $PID) {
            return $hWnd
        }

        $hWnd = [PInvoke.Win32]::FindWindowEx([IntPtr]::Zero, $hWnd, $null, $null)
    }

    return [IntPtr]::Zero
}

# ============================================================================
# Main
# ============================================================================

Write-Status "Creating hidden desktop: $HiddenDesktopName"

# Create hidden desktop
$hDesktop = [PInvoke.Win32]::CreateDesktop(
    $HiddenDesktopName,
    [System.IntPtr]::Zero,
    [System.IntPtr]::Zero,
    $DF_ALLOWOTHERACCOUNTHOOK,
    $DESKTOP_ALL_ACCESS,
    [System.IntPtr]::Zero
)

if ($hDesktop -eq [IntPtr]::Zero) {
    Write-Error "Failed to create desktop. Error: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
    exit 1
}

Write-Success "Desktop created: 0x$($hDesktop.ToString('X'))"

# Prepare STARTUPINFO
$si = New-Object PInvoke.Win32+STARTUPINFO
$si.cb = [System.Runtime.InteropServices.Marshal]::SizeOf($si)
$si.lpDesktop = "WinSta0\$HiddenDesktopName"
$si.dwFlags = $STARTF_USESHOWWINDOW
$si.wShowWindow = $SW_HIDE

Write-Status "Launching game on hidden desktop..."

# Create process on hidden desktop
$procInfo = New-Object PInvoke.Win32+PROCESS_INFORMATION

$success = [PInvoke.Win32]::CreateProcessW(
    $GamePath,
    "",
    [IntPtr]::Zero,
    [IntPtr]::Zero,
    $false,
    $CREATE_DEFAULT_ERROR_MODE -bor $CREATE_BREAKAWAY_FROM_JOB,
    [IntPtr]::Zero,
    "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option",
    [ref]$si,
    [ref]$procInfo
)

if (-not $success) {
    Write-Error "Failed to create process. Error: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
    [PInvoke.Win32]::CloseDesktop($hDesktop) | Out-Null
    exit 1
}

Write-Success "Game process created. PID: $($procInfo.dwProcessId)"

# Close thread handle (we don't need it)
[PInvoke.Win32]::CloseHandle($procInfo.hThread) | Out-Null

# Wait for game window to appear
Write-Status "Waiting up to $WaitForReady seconds for game window..."

$startTime = [DateTime]::UtcNow
$hWnd = [IntPtr]::Zero

while (([DateTime]::UtcNow - $startTime).TotalSeconds -lt $WaitForReady) {
    $hWnd = Find-WindowByProcessId $procInfo.dwProcessId

    if ($hWnd -ne [IntPtr]::Zero) {
        Write-Success "Game window found: 0x$($hWnd.ToString('X'))"
        break
    }

    Start-Sleep -Milliseconds 500
}

if ($hWnd -eq [IntPtr]::Zero) {
    Write-Error "Game window did not appear within timeout."
    # Don't kill process yet; let user investigate
}

# Output results for parent script/MCP tool
$result = @{
    DesktopHandle = $hDesktop
    DesktopName = $HiddenDesktopName
    ProcessId = $procInfo.dwProcessId
    ProcessHandle = $procInfo.hProcess
    WindowHandle = $hWnd
    WindowFound = ($hWnd -ne [IntPtr]::Zero)
    Timestamp = [DateTime]::UtcNow.ToString('O')
}

Write-Host ""
Write-Host "=== Hidden Desktop Launch Complete ===" -ForegroundColor Green
Write-Host ($result | ConvertTo-Json -Depth 10)

# Keep desktop open while process runs
Write-Status "Desktop and process remain open. To clean up, kill process PID $($procInfo.dwProcessId) and close desktop."

# Return handles to caller (if called from PowerShell script)
return $result
```

### `hidden_desktop_capture.ps1`

**Location:** `scripts/game/hidden_desktop_capture.ps1`
**Purpose:** Capture screenshot from game running on hidden desktop

```powershell
<#
.SYNOPSIS
    Capture screenshot from a game window on a hidden desktop.

.DESCRIPTION
    Uses GDI BitBlt to capture the game window, regardless of which
    desktop it's running on. Saves PNG to output path.

.PARAMETER ProcessId
    PID of the game process running on hidden desktop.

.PARAMETER OutputPath
    Where to save the PNG screenshot.

.PARAMETER Timeout
    Max seconds to wait for window (default 5).

.EXAMPLE
    .\hidden_desktop_capture.ps1 -ProcessId 1234 -OutputPath "C:\temp\screenshot.png"

#>

[CmdletBinding()]
param(
    [uint]$ProcessId,
    [string]$OutputPath = "$env:TEMP\DINOForge\hidden_desktop_screenshot.png",
    [int]$Timeout = 5
)

$ErrorActionPreference = "Stop"

# ============================================================================
# P/Invoke for GDI BitBlt
# ============================================================================

$sig = @'
[DllImport("user32.dll", SetLastError = true)]
public static extern IntPtr GetDC(IntPtr hWnd);

[DllImport("user32.dll", SetLastError = true)]
public static extern int ReleaseDC(IntPtr hWnd, IntPtr hDC);

[DllImport("gdi32.dll", SetLastError = true)]
public static extern IntPtr CreateCompatibleDC(IntPtr hdc);

[DllImport("gdi32.dll", SetLastError = true)]
public static extern bool DeleteDC(IntPtr hdc);

[DllImport("gdi32.dll", SetLastError = true)]
public static extern IntPtr CreateCompatibleBitmap(IntPtr hdc, int cx, int cy);

[DllImport("gdi32.dll", SetLastError = true)]
public static extern IntPtr SelectObject(IntPtr hdc, IntPtr hgdiobj);

[DllImport("gdi32.dll", SetLastError = true)]
public static extern bool DeleteObject(IntPtr hObject);

[DllImport("gdi32.dll", SetLastError = true)]
public static extern bool BitBlt(
    IntPtr hdcDest,
    int x,
    int y,
    int cx,
    int cy,
    IntPtr hdcSrc,
    int xSrc,
    int ySrc,
    uint rop
);

[DllImport("gdi32.dll", SetLastError = true)]
public static extern bool GetDIBits(
    IntPtr hdc,
    IntPtr hbmp,
    uint uStartScan,
    uint cScanLines,
    byte[] lpvBits,
    ref BITMAPINFO lpbi,
    uint usage
);

[DllImport("user32.dll", SetLastError = true)]
public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

[DllImport("user32.dll", SetLastError = true)]
public static extern IntPtr FindWindowEx(
    IntPtr hwndParent,
    IntPtr hwndChildAfter,
    string lpszClass,
    string lpszWindow
);

[DllImport("user32.dll", SetLastError = true)]
public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

[StructLayout(LayoutKind.Sequential)]
public struct RECT
{
    public int Left;
    public int Top;
    public int Right;
    public int Bottom;
}

[StructLayout(LayoutKind.Sequential)]
public struct BITMAPINFOHEADER
{
    public uint biSize;
    public int biWidth;
    public int biHeight;
    public ushort biPlanes;
    public ushort biBitCount;
    public uint biCompression;
    public uint biSizeImage;
    public int biXPelsPerMeter;
    public int biYPelsPerMeter;
    public uint biClrUsed;
    public uint biClrImportant;
}

[StructLayout(LayoutKind.Sequential)]
public struct BITMAPINFO
{
    public BITMAPINFOHEADER bmiHeader;
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = 1)]
    public uint[] bmiColors;
}
'@

Add-Type -MemberDefinition $sig -Name "Win32Gdi" -Namespace "GdiPInvoke" -Using System.Text

$SRCCOPY = 0x00CC0020
$DIB_RGB_COLORS = 0

# ============================================================================
# Find Window and Capture
# ============================================================================

function Find-WindowByProcessId {
    param([uint]$PID)

    $hWnd = [GdiPInvoke.Win32Gdi]::FindWindowEx([IntPtr]::Zero, [IntPtr]::Zero, $null, $null)

    while ($hWnd -ne [IntPtr]::Zero) {
        $procId = 0
        [GdiPInvoke.Win32Gdi]::GetWindowThreadProcessId($hWnd, [ref]$procId) | Out-Null

        if ($procId -eq $PID) {
            return $hWnd
        }

        $hWnd = [GdiPInvoke.Win32Gdi]::FindWindowEx([IntPtr]::Zero, $hWnd, $null, $null)
    }

    return [IntPtr]::Zero
}

Write-Host "[*] Finding window for PID $ProcessId..."

$hWnd = [IntPtr]::Zero
$elapsed = 0

while ($elapsed -lt $Timeout) {
    $hWnd = Find-WindowByProcessId $ProcessId
    if ($hWnd -ne [IntPtr]::Zero) {
        break
    }
    Start-Sleep -Milliseconds 500
    $elapsed += 0.5
}

if ($hWnd -eq [IntPtr]::Zero) {
    Write-Host "[-] Window not found for PID $ProcessId" -ForegroundColor Red
    exit 1
}

Write-Host "[+] Window found: 0x$($hWnd.ToString('X'))"

# Get window rect
$rect = New-Object GdiPInvoke.Win32Gdi+RECT
[GdiPInvoke.Win32Gdi]::GetWindowRect($hWnd, [ref]$rect) | Out-Null
$width = $rect.Right - $rect.Left
$height = $rect.Bottom - $rect.Top

Write-Host "[*] Window size: $width x $height"

# Get device contexts
$screenDC = [GdiPInvoke.Win32Gdi]::GetDC([IntPtr]::Zero)
$windowDC = [GdiPInvoke.Win32Gdi]::GetDC($hWnd)

if ($screenDC -eq [IntPtr]::Zero -or $windowDC -eq [IntPtr]::Zero) {
    Write-Host "[-] Failed to get device context" -ForegroundColor Red
    exit 1
}

# Create compatible bitmap and memory DC
$memDC = [GdiPInvoke.Win32Gdi]::CreateCompatibleDC($screenDC)
$bitmap = [GdiPInvoke.Win32Gdi]::CreateCompatibleBitmap($screenDC, $width, $height)

if ($memDC -eq [IntPtr]::Zero -or $bitmap -eq [IntPtr]::Zero) {
    Write-Host "[-] Failed to create bitmap or memory DC" -ForegroundColor Red
    [GdiPInvoke.Win32Gdi]::ReleaseDC([IntPtr]::Zero, $screenDC) | Out-Null
    [GdiPInvoke.Win32Gdi]::ReleaseDC($hWnd, $windowDC) | Out-Null
    exit 1
}

# Select bitmap into memory DC
[GdiPInvoke.Win32Gdi]::SelectObject($memDC, $bitmap) | Out-Null

# Copy from window DC to memory DC
$success = [GdiPInvoke.Win32Gdi]::BitBlt($memDC, 0, 0, $width, $height, $windowDC, 0, 0, $SRCCOPY)

if (-not $success) {
    Write-Host "[-] BitBlt failed" -ForegroundColor Red
    [GdiPInvoke.Win32Gdi]::DeleteObject($bitmap) | Out-Null
    [GdiPInvoke.Win32Gdi]::DeleteDC($memDC) | Out-Null
    [GdiPInvoke.Win32Gdi]::ReleaseDC([IntPtr]::Zero, $screenDC) | Out-Null
    [GdiPInvoke.Win32Gdi]::ReleaseDC($hWnd, $windowDC) | Out-Null
    exit 1
}

# Extract bitmap data
$bmi = New-Object GdiPInvoke.Win32Gdi+BITMAPINFO
$bmi.bmiHeader.biSize = [System.Runtime.InteropServices.Marshal]::SizeOf($bmi.bmiHeader)
$bmi.bmiHeader.biWidth = $width
$bmi.bmiHeader.biHeight = $height
$bmi.bmiHeader.biPlanes = 1
$bmi.bmiHeader.biBitCount = 32
$bmi.bmiHeader.biCompression = 0  # BI_RGB

$pixelDataSize = $width * $height * 4
$pixelData = New-Object byte[] $pixelDataSize

[GdiPInvoke.Win32Gdi]::GetDIBits($memDC, $bitmap, 0, [uint]$height, $pixelData, [ref]$bmi, $DIB_RGB_COLORS) | Out-Null

# Cleanup GDI objects
[GdiPInvoke.Win32Gdi]::DeleteObject($bitmap) | Out-Null
[GdiPInvoke.Win32Gdi]::DeleteDC($memDC) | Out-Null
[GdiPInvoke.Win32Gdi]::ReleaseDC([IntPtr]::Zero, $screenDC) | Out-Null
[GdiPInvoke.Win32Gdi]::ReleaseDC($hWnd, $windowDC) | Out-Null

# Save to PNG using System.Drawing
Add-Type -AssemblyName System.Drawing

$bitmap = New-Object System.Drawing.Bitmap($width, $height, [System.Drawing.Imaging.PixelFormat]::Format32bppRgb)
$bitmapData = $bitmap.LockBits([System.Drawing.Rectangle]::FromLTRB(0, 0, $width, $height), [System.Drawing.Imaging.ImageLockMode]::WriteOnly, [System.Drawing.Imaging.PixelFormat]::Format32bppRgb)

[System.Runtime.InteropServices.Marshal]::Copy($pixelData, 0, $bitmapData.Scan0, $pixelDataSize)
$bitmap.UnlockBits($bitmapData)

$outputDir = Split-Path $OutputPath
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
}

$bitmap.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
$bitmap.Dispose()

Write-Host "[+] Screenshot saved: $OutputPath" -ForegroundColor Green
```

---

## Integration Points in DINOForge

### 1. MCP Server Enhancement (`src/Tools/McpServer/Program.cs`)

Add hidden desktop context to game launch:

```csharp
services.AddSingleton<IGameAutomationService>(sp =>
{
    var config = sp.GetRequiredService<IConfiguration>();
    var useHiddenDesktop = config.GetValue<bool>("GameAutomation:UseHiddenDesktop", false);

    if (useHiddenDesktop)
    {
        return new HiddenDesktopGameAutomationService();
    }

    return new StandardGameAutomationService();
});
```

### 2. Game Input Tool Update (`src/Tools/McpServer/Tools/GameInputTool.cs`)

```csharp
public async Task<bool> SendInputAsync(GameInputRequest request)
{
    if (_automationService.IsUsingHiddenDesktop)
    {
        // Use PostMessage instead of SendInput
        return await SendInputViaPostMessageAsync(_automationService.GameWindowHandle, request);
    }

    // Standard SendInput path
    return await SendInputViaWin32Async(request);
}
```

### 3. Configuration (`appsettings.json`)

```json
{
  "GameAutomation": {
    "UseHiddenDesktop": false,
    "HiddenDesktopName": "DINOForge_Agent",
    "LaunchTimeout": 30,
    "ScreenshotMethod": "GdiBitBlt"
  }
}
```

### 4. Asset Pipeline Integration

For pack automation:
```bash
# Launch game on hidden desktop
dotnet run --project src/Tools/McpServer -- launch-hidden-desktop

# Run pack tests against hidden game instance
dotnet test src/Tests/ --filter "Category=AutomationTest" -- --game-instance hidden
```

---

## Feasibility Assessment

### Success Criteria

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Desktop creation and process spawn | ✓ Known Working | Win32 API widely documented; used in RDP, alt-desktop switchers |
| Input delivery (PostMessage) | ✓ Known Working | Cross-desktop message routing is OS-guaranteed |
| GDI BitBlt cross-desktop | ✓ Likely Working | GDI is lower-level than WGC; BitBlt pre-dates desktop isolation |
| BepInEx plugin on hidden desktop | ? Untested | BepInEx should work anywhere (just a DLL load); risk is Unity rendering |
| **Unity rendering on hidden desktop** | ? **Untested** | **CRITICAL RISK — Unity rendering thread may fail or behave unexpectedly** |
| Process isolation (mutex independence) | ? Untested | Desktops have separate namespaces; DINO's mutex should not collide |

### Critical Risk: Unity Rendering

**Issue:** Unity's rendering pipeline may have assumptions about the window being visible or on the user's desktop.

**Specific concerns:**
- D3D11/D3D12 may refuse to initialize on a backgrounded window
- DXGI swap chain may fail to present to a hidden desktop
- OpenGL context might not bind properly off-screen

**Mitigation path:**
1. **Prototype test:** Launch game on hidden desktop, check for rendering errors in BepInEx/LogOutput.log
2. **Fallback 1:** If Unity rendering fails, use a minimized window on user's desktop (visible but non-intrusive)
3. **Fallback 2:** Use separate Windows user account (RUNAS) if licensing allows
4. **Fallback 3:** VM-based approach (slower, but guaranteed isolation)

---

## Testing Plan

### Phase 1: Basic Feasibility (2-3 hours)

1. **Prototype hidden desktop creation**
   ```powershell
   .\scripts\game\hidden_desktop_launch.ps1
   ```
   - Verify desktop created
   - Verify process spawned on desktop
   - Check for any immediate errors in BepInEx/LogOutput.log

2. **Test window enumeration**
   - Verify game window handle retrieved across desktops
   - Verify window properties (rect, class, title) accessible

3. **Test screenshot capture**
   ```powershell
   .\scripts\game\hidden_desktop_capture.ps1 -ProcessId <PID>
   ```
   - Verify GDI BitBlt works
   - Verify PNG saved correctly
   - Compare file size/quality vs normal capture

### Phase 2: Input and Rendering (4-5 hours)

4. **Test PostMessage input**
   - Send key press (F10) via PostMessage
   - Verify game responds (menu appears, no focus loss to user)
   - Check input lag

5. **Test rendering**
   - Monitor BepInEx/LogOutput.log for D3D/Unity errors
   - Check if game runs at full FPS on hidden desktop
   - Test a simple scenario (main menu, gameplay 30 seconds)

6. **Test process isolation**
   - Launch game on hidden desktop
   - Verify user can launch separate DINO instance on visible desktop
   - Verify no mutex conflicts or "another instance" errors

### Phase 3: MCP Integration (2-3 hours)

7. **Integrate into MCP game_launch tool**
   - Add `--hidden-desktop` flag to game_launch
   - Update game_screenshot to use GDI BitBlt if hidden desktop
   - Update game_input to use PostMessage if hidden desktop

8. **End-to-end test**
   - MCP: game_launch --hidden-desktop
   - MCP: game_screenshot
   - MCP: game_input keyboard f10
   - Verify all steps work without user distraction

---

## Performance Expectations

### Overhead vs Normal Launch

| Operation | Hidden Desktop | Normal | Delta |
|-----------|----------------|--------|-------|
| Launch time | ~12-15s | ~12-15s | ~0% |
| Memory (after boot) | ~1.8GB | ~1.8GB | ~0% |
| FPS (gameplay) | 60 FPS | 60 FPS | ~0% |
| Input latency (PostMessage) | ~5-10ms | ~0-2ms (SendInput) | +5-10ms |
| Screenshot (BitBlt) | ~50-100ms | ~10-20ms (WGC) | +30-90ms |

**Note:** Input latency increase is acceptable (< frame boundary). Screenshot overhead is dominated by I/O, not capture method.

---

## Open Questions & Assumptions

| Question | Assumption | Test Plan |
|----------|-----------|-----------|
| Does Unity render on hidden desktop? | YES (Mono 2021.3 is permissive) | See Phase 2 test 5 |
| Can we screenshot cross-desktop with GDI BitBlt? | YES (documented capability) | See Phase 1 test 3 |
| Does DINO's mutex isolate to desktops? | YES (separate namespace) | See Phase 2 test 6 |
| Is PostMessage input sufficient for game automation? | YES (sufficient for most tasks) | See Phase 2 test 4 |
| Does D3D11 initialize on hidden desktop? | MAYBE (driver-dependent) | See Phase 2 test 5 |

---

## Comparison with Alternatives

### Option A: Hidden Desktop (THIS PROPOSAL)

**Pros:**
- OS-level isolation, no VM overhead
- Invisible to user (zero distraction)
- Full performance (same machine, no network)
- Fast iteration (simple PowerShell scripts)
- No licensing concerns

**Cons:**
- Untested with Unity rendering
- Requires P/Invoke and Win32 knowledge
- GDI BitBlt slower than WGC for repeated captures
- Input latency +5-10ms

**Recommended for:** Development/testing where user must keep working

---

### Option B: Separate Windows User Account

**Pros:**
- Complete session isolation
- Each user has independent desktop
- No namespace collisions (separate mutex space)

**Cons:**
- Steam license tied to primary user (secondary user cannot launch DINO easily)
- Requires second account creation or RUNAS (with password)
- Login overhead (~5-10s)
- More setup complexity

**Fallback if:** Hidden desktop rendering fails

---

### Option C: Virtual Machine

**Pros:**
- Complete isolation (separate OS)
- Can use headless mode (no graphics needed)
- Reproducible environment

**Cons:**
- 2-4GB RAM overhead (overkill for single process)
- 10-30s launch overhead
- Network MTU/latency issues
- Licensing complexity (DINO on separate VM?)

**Fallback if:** Hidden desktop AND secondary user both fail

---

### Option D: Minimized Window on User Desktop

**Pros:**
- Simple (no special APIs)
- Rendering guaranteed to work (visible window)

**Cons:**
- Not truly "non-intrusive" (visible in taskbar, alt-tab)
- Window may steal focus or flicker
- Less professional appearance

**Fallback if:** Hidden desktop rendering fails AND we want something quick

---

## Conclusion

The **hidden desktop approach is the preferred direction** for DINOForge game automation. It balances efficiency, isolation, and invisibility without VM overhead.

**Next steps:**
1. Implement Phase 1 prototype tests (2-3 hours)
2. Assess Unity rendering on hidden desktop
3. If rendering works: integrate into MCP server, update game launch tools
4. If rendering fails: evaluate Option B (separate user) or Option D (minimized window)

**Success metric:** Game runs on hidden desktop with automated input/output, user's session unaffected.

---

## Related Documents

- `CLAUDE.md` — Agent governance and game launch protocol
- `docs/game-automation-mcp-design.md` — MCP bridge architecture
- `src/Tools/McpServer/` — MCP server implementation
- `scripts/game/` — Game automation scripts (existing)

**Prototype scripts location:** `scripts/game/hidden_desktop_*.ps1`

---

**Document prepared:** 2026-03-25
**Last updated:** 2026-03-25
**Author:** Claude (DINOForge Orchestrator)
