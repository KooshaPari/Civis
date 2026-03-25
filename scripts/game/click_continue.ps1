Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd); [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo); [DllImport("user32.dll")] public static extern void mouse_event(int dwFlags, int dx, int dy, int dwData, int dwExtraInfo); [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);' -Name CLICKER -Namespace System

$proc = Get-Process 'Diplomacy*' -ErrorAction SilentlyContinue
if (-not $proc) { Write-Host "Game not running"; exit 1 }
$hwnd = $proc.MainWindowHandle
[CLICKER]::SetForegroundWindow($hwnd)
Start-Sleep -Milliseconds 800

# Press Escape 8x to close all dialogs
for ($i = 0; $i -lt 8; $i++) {
    [CLICKER]::keybd_event(0x1B, 0, 0, 0)
    Start-Sleep -Milliseconds 150
    [CLICKER]::keybd_event(0x1B, 0, 2, 0)
    Start-Sleep -Milliseconds 350
}
Start-Sleep -Milliseconds 500
Write-Host "Dialogs dismissed"

# Click CONTINUE (y~295 from top of 1680x1050 window)
[CLICKER]::SetCursorPos(120, 295)
Start-Sleep -Milliseconds 300
[CLICKER]::mouse_event(2, 0, 0, 0, 0)
Start-Sleep -Milliseconds 100
[CLICKER]::mouse_event(4, 0, 0, 0, 0)
Write-Host "Clicked CONTINUE at (120, 295)"
