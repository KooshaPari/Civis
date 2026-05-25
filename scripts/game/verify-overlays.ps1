# DINOForge overlay verification script
# Sends F9/F10 to game window, takes screenshots
param(
    [string]$GameTitle = "Diplomacy is Not an Option"
)

Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win32 {
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);
    [DllImport("user32.dll")] public static extern short GetAsyncKeyState(int vKey);
    [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    public const int VK_F9 = 0x78;
    public const int VK_F10 = 0x79;
    public const uint KEYEVENTF_KEYUP = 0x0002;
}
"@

$hwnd = [Win32]::FindWindow([NullString]::Value, $GameTitle)
if ($hwnd -eq [IntPtr]::Zero) {
    Write-Host "[ERROR] Game window not found: $GameTitle"
    exit 1
}

Write-Host "[OK] Game window found, HWND=$hwnd"

# Bring to foreground
[Win32]::SetForegroundWindow($hwnd) | Out-Null
Start-Sleep -Milliseconds 200

function Send-FKey {
    param([int]$vk, [string]$name)
    Write-Host "Sending $name..."
    [Win32]::keybd_event($vk, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 100
    [Win32]::keybd_event($vk, 0, [Win32]::KEYEVENTF_KEYUP, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 500
}

function Get-Screenshot {
    param([string]$Label)
    Add-Type -AssemblyName System.Windows.Forms
    Add-Type -AssemblyName System.Drawing
    $bounds = [System.Drawing.Rectangle]::FromLTRB(0, 0, 1920, 1080)
    $bmp = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
    $path = "$env:TEMP\DINOForge\overlay_${Label}_$(Get-Date -Format 'HHmmss').png"
    $bmp.Save($path)
    $g.Dispose()
    $bmp.Dispose()
    Write-Host "[Screenshot] $path"
    return $path
}

Write-Host "--- Testing F9 (Debug Overlay) ---"
Send-FKey -vk ([Win32]::VK_F9) -name "F9"
Get-Screenshot -Label "f9"

Write-Host "--- Testing F10 (Mod Menu) ---"
Send-FKey -vk ([Win32]::VK_F10) -name "F10"
Get-Screenshot -Label "f10"

Write-Host "--- Done. Screenshots saved to `$env:TEMP\DINOForge\ ---"
