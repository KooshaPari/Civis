Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd); [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo); [DllImport("user32.dll")] public static extern void mouse_event(int dwFlags, int dx, int dy, int dwData, int dwExtraInfo); [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);' -Name PRESSCONT -Namespace System

$proc = Get-Process 'Diplomacy*' -ErrorAction SilentlyContinue
$hwnd = $proc.MainWindowHandle
[PRESSCONT]::SetForegroundWindow($hwnd)
Start-Sleep -Milliseconds 800

# First dismiss DINOForge with F10 if open
[PRESSCONT]::keybd_event(0x79, 0, 0, 0); Start-Sleep -Milliseconds 100; [PRESSCONT]::keybd_event(0x79, 0, 2, 0)
Start-Sleep -Milliseconds 400

# Escape to close any overlay/dialog
for ($i = 0; $i -lt 5; $i++) {
    [PRESSCONT]::keybd_event(0x1B, 0, 0, 0); Start-Sleep -Milliseconds 150; [PRESSCONT]::keybd_event(0x1B, 0, 2, 0)
    Start-Sleep -Milliseconds 300
}
Start-Sleep -Milliseconds 600
Write-Host "Pressed Escape x5"

# Now click in the main menu area at top-left (AWAY from DINOForge panel area)
# The main menu left column: CONTINUE is at roughly x=90, y=284
# Scan multiple y values to find CONTINUE (try y from 280-310)
[PRESSCONT]::SetCursorPos(90, 284); Start-Sleep -Milliseconds 300
[PRESSCONT]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 100; [PRESSCONT]::mouse_event(4, 0, 0, 0, 0)
Write-Host "Clicked at (90, 284)"
