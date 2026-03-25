Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd); [DllImport("user32.dll")] public static extern void mouse_event(int dwFlags, int dx, int dy, int dwData, int dwExtraInfo); [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);' -Name LOADER -Namespace System

$proc = Get-Process 'Diplomacy*' -ErrorAction SilentlyContinue
if (-not $proc) { Write-Host "Game not running"; exit 1 }
[LOADER]::SetForegroundWindow($proc.MainWindowHandle)
Start-Sleep -Milliseconds 600

# Close any popup in the top-right corner of screen
[LOADER]::SetCursorPos(1655, 15)
Start-Sleep -Milliseconds 200
[LOADER]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 100; [LOADER]::mouse_event(4, 0, 0, 0, 0)
Start-Sleep -Milliseconds 500

# Click first save "OFF WE GO..." at y=315, x=840 (center of the save row)
# But this time click on the RIGHT side, away from DINOForge dialog (x=1000)
[LOADER]::SetCursorPos(1000, 315)
Start-Sleep -Milliseconds 200
[LOADER]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 100; [LOADER]::mouse_event(4, 0, 0, 0, 0)
Start-Sleep -Milliseconds 300
Write-Host "Clicked save row at (1000, 315)"

# Wait and check for a Load button to appear
Start-Sleep -Milliseconds 1000

# If a Load button appeared, it's usually on the right side of selected save
# Try clicking the star/load button at right edge of save row ~x=1360, y=315
[LOADER]::SetCursorPos(1360, 315)
Start-Sleep -Milliseconds 200
[LOADER]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 100; [LOADER]::mouse_event(4, 0, 0, 0, 0)
Write-Host "Clicked load button at (1360, 315)"
