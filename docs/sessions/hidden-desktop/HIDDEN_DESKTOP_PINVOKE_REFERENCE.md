# Hidden Desktop Test — P/Invoke Reference

This document provides detailed reference for the Win32 P/Invoke calls used in the hidden desktop rendering test.

## Overview

The test uses four main P/Invoke modules:

| Module | Purpose | Key APIs |
|--------|---------|----------|
| `Win32Desktop` | Desktop lifecycle management | CreateDesktop, CloseDesktop, SetThreadDesktop |
| `Win32Process` | Process creation & termination | CreateProcess, TerminateProcess |
| `Win32Window` | Window enumeration & properties | FindWindow, GetWindowRect, IsWindow |
| `Win32Gdi` | Screen capture via device contexts | GetDC, CreateCompatibleDC, BitBlt, SelectObject |

---

## Module: Win32Desktop

**Purpose**: Create, switch to, and destroy hidden desktops.

### CreateDesktop

```csharp
[DllImport("user32.dll", SetLastError=true)]
public static extern IntPtr CreateDesktop(
    string lpszDesktop,         // Desktop name (e.g., "DINOForge_Test")
    IntPtr lpszDevice,          // Device name (almost always NULL)
    IntPtr pDevMode,            // Display mode (almost always NULL)
    uint dwFlags,               // Flags (usually 0 for hidden)
    uint dwDesiredAccess,       // Access rights (DESKTOP_ALL_ACCESS)
    IntPtr lpsa                 // Security attributes (NULL for default)
);

// Returns: IntPtr handle to desktop, or IntPtr.Zero on failure
// LastWin32Error: Contains error code if failed
```

**Usage in test**:
```powershell
$desktopHandle = [Win32Desktop]::CreateDesktop(
    "DINOForge_Test_1234",          # Random-suffixed name
    [IntPtr]::Zero,                 # No device override
    [IntPtr]::Zero,                 # No display mode override
    0,                              # Flags: 0 = hidden
    [Win32Desktop]::DESKTOP_ALL_ACCESS,  # Full access
    [IntPtr]::Zero                  # Default security
)

if ($desktopHandle -eq [IntPtr]::Zero) {
    $err = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
    Write-Error "CreateDesktop failed: $err"
}
```

**Key points**:
- Desktop name can be arbitrary; test uses "DINOForge_Test_XXXX" (unique per run)
- When `dwFlags = 0`, desktop is **invisible** (not shown on screen)
- Returns valid IntPtr on success, IntPtr.Zero on failure
- Common failure: error 1314 (insufficient privilege) — need elevation
- Desktop remains valid until `CloseDesktop()` is called

### CloseDesktop

```csharp
[DllImport("user32.dll", SetLastError=true)]
public static extern bool CloseDesktop(IntPtr hDesktop);

// Returns: true if successful, false otherwise
```

**Usage in test**:
```powershell
[Win32Desktop]::CloseDesktop($desktopHandle) | Out-Null
```

**Key points**:
- Must be called in cleanup to avoid resource leak
- Should be called after all processes on the desktop are terminated
- Trying to close an invalid handle returns false but doesn't crash

### GetThreadDesktop & SetThreadDesktop

**Not used in this test** (but included for reference).

These APIs are for thread-to-desktop affinity. The test doesn't use them because the game process is launched directly onto the desktop via `CreateProcess(STARTUPINFO.lpDesktop)`.

---

## Module: Win32Process

**Purpose**: Launch game process and manage its lifecycle.

### CreateProcess

```csharp
[DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)]
public static extern bool CreateProcess(
    string lpApplicationName,       // Exe path (NULL if in lpCommandLine)
    string lpCommandLine,           // Command line arguments
    IntPtr lpProcessAttributes,     // Process security attributes
    IntPtr lpThreadAttributes,      // Thread security attributes
    bool bInheritHandles,           // Inherit handles? (usually false)
    uint dwCreationFlags,           // Flags (CREATE_NO_WINDOW, priority, etc.)
    IntPtr lpEnvironment,           // Environment variables (NULL for inherit)
    string lpCurrentDirectory,      // Working directory
    ref STARTUPINFO lpStartupInfo,  // ← KEY: contains lpDesktop
    out PROCESS_INFORMATION lpProcessInformation  // Returns PID, thread handles
);

// Returns: true if successful, false otherwise
```

**The STARTUPINFO struct**:
```csharp
[StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)]
public struct STARTUPINFO {
    public int cb;              // Size of struct (required)
    public string lpReserved;   // Reserved
    public string lpDesktop;    // ← CRITICAL: "WinSta0\DINOForge_Test"
    public string lpTitle;      // Window title (optional)
    public int dwX;             // Window X position
    public int dwY;             // Window Y position
    public int dwXSize;         // Window width
    public int dwYSize;         // Window height
    public int dwXCountChars;   // Console columns
    public int dwYCountChars;   // Console rows
    public int dwFillAttribute; // Console fill attr
    public int dwFlags;         // Flags (STARTF_USESTDHANDLES, etc.)
    public short wShowWindow;   // Show mode (SW_HIDE, SW_SHOW, etc.)
    public short cbReserved2;   // Reserved
    public IntPtr lpReserved2;  // Reserved
    public IntPtr hStdInput;    // Standard input handle
    public IntPtr hStdOutput;   // Standard output handle
    public IntPtr hStdError;    // Standard error handle
}
```

**Usage in test**:
```powershell
$startupInfo = New-Object Win32Process+STARTUPINFO
$startupInfo.cb = [System.Runtime.InteropServices.Marshal]::SizeOf($startupInfo)
$startupInfo.lpDesktop = "WinSta0\DINOForge_Test_1234"  # ← Key line
$startupInfo.dwFlags = 0x00000100  # STARTF_USESTDHANDLES

$processInfo = New-Object Win32Process+PROCESS_INFORMATION

$success = [Win32Process]::CreateProcess(
    "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
    "",
    [IntPtr]::Zero,
    [IntPtr]::Zero,
    $false,
    [Win32Process]::CREATE_NO_WINDOW -bor [Win32Process]::NORMAL_PRIORITY_CLASS,
    [IntPtr]::Zero,
    "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option",
    [ref]$startupInfo,
    [ref]$processInfo
)

if ($success) {
    Write-Host "Launched PID: $($processInfo.dwProcessId)"
}
```

**Key points**:
- **lpDesktop is the magic parameter**: sets which desktop the process runs on
- Format: `"WinSta0\<desktop_name>"` where WinSta0 is the standard window station
- CREATE_NO_WINDOW suppresses console window (game will still render to its own window)
- Process handle and thread handle must be closed via `CloseHandle()` to avoid leak

### TerminateProcess

```csharp
[DllImport("kernel32.dll", SetLastError=true)]
public static extern bool TerminateProcess(IntPtr hProcess, uint uExitCode);

// Returns: true if successful, false otherwise
// Note: Exit code (second param) is what killed process reports to OS
```

**Usage in test**:
```powershell
[Win32Process]::TerminateProcess($gameProcess.ProcessHandle, 0) | Out-Null
```

**Key points**:
- Force-kills the process immediately (does not allow graceful shutdown)
- Safe to call even if process already dead (returns false but doesn't crash)
- Test also uses `Stop-Process -Force` as backup for extra safety

### CloseHandle

```csharp
[DllImport("kernel32.dll", SetLastError=true)]
public static extern bool CloseHandle(IntPtr hObject);

// Returns: true if successful, false otherwise
```

**Usage in test**:
```powershell
[Win32Process]::CloseHandle($processInfo.hThread)
[Win32Process]::CloseHandle($processInfo.hProcess)
```

**Key points**:
- Must be called for every handle returned by CreateProcess
- Calling CloseHandle on invalid handle returns false but is safe
- Leak if not called: process handle stays open in kernel

---

## Module: Win32Window

**Purpose**: Find game window and get its properties.

### FindWindow

```csharp
[DllImport("user32.dll", CharSet=CharSet.Auto, SetLastError=true)]
public static extern IntPtr FindWindow(
    string lpClassName,     // Window class name (NULL to ignore)
    string lpWindowName     // Window title text to search for
);

// Returns: Window handle (HWND), or IntPtr.Zero if not found
```

**Usage in test**:
```powershell
$windowHandle = [Win32Window]::FindWindow([IntPtr]::Zero, "Diplomacy is Not an Option")

if ($windowHandle -eq [IntPtr]::Zero) {
    Write-Error "Window not found"
} else {
    Write-Host "Found window: $windowHandle"
}
```

**Key points**:
- Searches all visible (and hidden!) windows for matching title
- Game's main window title is "Diplomacy is Not an Option"
- Returns immediately; call this in a loop to wait for window creation
- Works even for windows on hidden desktops (great for our use case!)

### GetWindowRect

```csharp
[DllImport("user32.dll", SetLastError=true)]
public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

[StructLayout(LayoutKind.Sequential)]
public struct RECT {
    public int Left;
    public int Top;
    public int Right;
    public int Bottom;
}

// Returns: true if successful, false otherwise
// lpRect is filled with window bounds in screen coordinates
```

**Usage in test**:
```powershell
$rect = New-Object Win32Window+RECT
[Win32Window]::GetWindowRect($windowHandle, [ref]$rect)

$width = $rect.Right - $rect.Left
$height = $rect.Bottom - $rect.Top

Write-Host "Window size: ${width}x${height}"
```

**Key points**:
- Returns coordinates in screen coordinates (useful for BitBlt)
- Can return zero-size rect for minimized/invalid windows (test checks for this)
- Top-left is usually (0,0) for fullscreen, but may vary for windowed

### IsWindow

```csharp
[DllImport("user32.dll", SetLastError=true)]
public static extern bool IsWindow(IntPtr hWnd);

// Returns: true if window is valid, false otherwise
```

**Usage in test**:
```powershell
if ([Win32Window]::IsWindow($windowHandle)) {
    Write-Host "Window is still valid"
} else {
    Write-Error "Window was destroyed"
}
```

**Key points**:
- Quick validity check before using window handle
- Returns false if handle is stale/invalid
- Used in polling loop to verify window hasn't closed

---

## Module: Win32Gdi

**Purpose**: Capture game framebuffer via GDI BitBlt.

### GetDC

```csharp
[DllImport("user32.dll", SetLastError=true)]
public static extern IntPtr GetDC(IntPtr hWnd);

// Returns: Device context (HDC) handle, or IntPtr.Zero on failure
// Device context is used for all subsequent GDI drawing/capture operations
```

**Usage in test**:
```powershell
$srcDC = [Win32Gdi]::GetDC($windowHandle)

if ($srcDC -eq [IntPtr]::Zero) {
    Write-Error "Failed to get window DC"
} else {
    Write-Host "Got device context: $srcDC"
}
```

**Key points**:
- GetDC retrieves the device context for a window
- Works for ANY window, including on hidden desktops
- Must call `ReleaseDC()` when done (see below)
- Returns handle to window's framebuffer

### CreateCompatibleDC

```csharp
[DllImport("gdi32.dll", SetLastError=true)]
public static extern IntPtr CreateCompatibleDC(IntPtr hdc);

// Returns: New device context handle, or IntPtr.Zero on failure
// The new DC is compatible with the input DC
```

**Usage in test**:
```powershell
$dstDC = [Win32Gdi]::CreateCompatibleDC($srcDC)

if ($dstDC -eq [IntPtr]::Zero) {
    Write-Error "Failed to create compatible DC"
}
```

**Key points**:
- Creates an in-memory device context
- "Compatible" means it can accept the same drawing operations as the source
- Used as destination for BitBlt copy operation
- Must call `DeleteDC()` when done

### CreateCompatibleBitmap

```csharp
[DllImport("gdi32.dll", SetLastError=true)]
public static extern IntPtr CreateCompatibleBitmap(IntPtr hdc, int nWidth, int nHeight);

// Returns: Bitmap handle (HBITMAP), or IntPtr.Zero on failure
```

**Usage in test**:
```powershell
$hBmp = [Win32Gdi]::CreateCompatibleBitmap($srcDC, $width, $height)

if ($hBmp -eq [IntPtr]::Zero) {
    Write-Error "Failed to create bitmap"
}
```

**Key points**:
- Allocates memory for bitmap of specified dimensions
- "Compatible" means same bit depth as source DC
- Will hold the copied framebuffer pixels
- Must call `DeleteObject()` when done

### SelectObject

```csharp
[DllImport("gdi32.dll", SetLastError=true)]
public static extern IntPtr SelectObject(IntPtr hdc, IntPtr hgdiobj);

// Returns: Previous object handle (to restore later), or IntPtr.Zero on failure
// Selects a bitmap/brush/pen into a device context
```

**Usage in test**:
```powershell
$oldBmp = [Win32Gdi]::SelectObject($dstDC, $hBmp)

# Now $dstDC is ready to receive BitBlt copy to $hBmp
```

**Key points**:
- Selects the bitmap into the compatible DC
- Without this, BitBlt would draw to the wrong target
- Must store the old object handle to restore it later (good practice)
- After SelectObject, the bitmap is "active" in the DC

### BitBlt

```csharp
[DllImport("gdi32.dll", SetLastError=true)]
public static extern bool BitBlt(
    IntPtr hdcDest,    // Destination device context
    int nXDest,        // X offset in destination
    int nYDest,        // Y offset in destination
    int nWidth,        // Width of copy region
    int nHeight,       // Height of copy region
    IntPtr hdcSrc,     // Source device context
    int nXSrc,         // X offset in source
    int nYSrc,         // Y offset in source
    uint dwRop         // Raster operation (usually SRCCOPY)
);

// Returns: true if successful, false otherwise
// This is the KEY function: copies pixels from one DC to another
```

**Usage in test**:
```powershell
[Win32Gdi]::BitBlt(
    $dstDC,            # Copy INTO memory DC
    0, 0,              # At offset (0,0)
    $width, $height,   # Full size
    $srcDC,            # Copy FROM window DC
    0, 0,              # From offset (0,0)
    [Win32Gdi]::SRCCOPY  # SRCCOPY = straight copy (no transparency, inversion, etc.)
)
```

**Key points**:
- This is the magic function: copies game framebuffer to memory
- SRCCOPY (0x00CC0020) = simple copy (source to dest, no special effects)
- Works even for windows on hidden desktops
- Result: memory DC now contains game frame pixels

### DeleteDC & DeleteObject

```csharp
[DllImport("gdi32.dll", SetLastError=true)]
public static extern bool DeleteDC(IntPtr hdc);

[DllImport("gdi32.dll", SetLastError=true)]
public static extern bool DeleteObject(IntPtr hObject);
```

**Usage in test**:
```powershell
[Win32Gdi]::SelectObject($dstDC, $oldBmp)  # Restore old bitmap
[Win32Gdi]::DeleteObject($hBmp)            # Free bitmap memory
[Win32Gdi]::DeleteDC($dstDC)               # Free DC memory
[Win32Gdi]::ReleaseDC($windowHandle, $srcDC)  # Release window DC
```

**Key points**:
- Must be called in cleanup to avoid resource leaks
- DeleteDC for device contexts, DeleteObject for bitmaps/brushes/pens
- ReleaseDC for device contexts obtained via GetDC
- Safe to call on invalid handles (returns false, doesn't crash)

### GetPixel

```csharp
// Note: Not explicitly defined in P/Invoke in the test
// Instead, we convert HBITMAP to managed System.Drawing.Bitmap
// and use managed .GetPixel() method

[System.Drawing.Bitmap]::FromHbitmap($hBmp).GetPixel(x, y)
```

**Usage in test**:
```powershell
$bmp = [System.Drawing.Bitmap]::FromHbitmap($hBmp)
$pixel = $bmp.GetPixel(100, 100)

if ($pixel.R -gt 10 -or $pixel.G -gt 10 -or $pixel.B -gt 10) {
    Write-Host "Pixel is not black"
}
```

**Key points**:
- Converts Win32 HBITMAP to managed .NET Bitmap
- Allows use of managed APIs for pixel inspection
- Test samples 5 locations and checks if non-black

---

## Data Flow: From Game to Screenshot

```
Game Window (D3D11 framebuffer)
         ↓
    [GetDC] → srcDC (window device context)
         ↓
[CreateCompatibleDC] → dstDC (memory device context)
         ↓
[CreateCompatibleBitmap] → hBmp (memory bitmap)
         ↓
[SelectObject] hBmp into dstDC
         ↓
[BitBlt] copy from srcDC to dstDC (pixels flow srcDC → hBmp)
         ↓
[Bitmap.FromHbitmap] → managed Bitmap object
         ↓
[Bitmap.Save] → PNG file on disk
         ↓
$env:TEMP\DINOForge\hidden_desktop_test.png
```

---

## Error Codes

| Code | Meaning | Common Cause |
|------|---------|--------------|
| 1314 | SE_DESKTOP_NAME privilege not held | Need elevation (run as admin) |
| 5 | Access denied | Insufficient permissions |
| 2 | File not found | Bad game path |
| 6 | Invalid handle | Handle is stale/invalid |
| 87 | Invalid parameter | Bad struct layout or parameter value |

**To get error code in PowerShell**:
```powershell
$err = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
Write-Error "LastWin32Error: $err"
```

---

## References

- **Microsoft Docs**: [Win32 API Reference](https://docs.microsoft.com/en-us/windows/win32/api/)
- **Desktop APIs**: [desktops.h](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createdesktopw)
- **Process APIs**: [processthreadsapi.h](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessa)
- **GDI Reference**: [wingdi.h](https://docs.microsoft.com/en-us/windows/win32/gdi/gdi-reference)
- **Pinvoke.net**: Community P/Invoke reference (useful for marshaling reference)

---

**Created**: 2026-03-25
**Maintained by**: DINOForge Development
