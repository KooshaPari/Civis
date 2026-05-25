Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd); [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo); [DllImport("user32.dll")] public static extern void mouse_event(int dwFlags, int dx, int dy, int dwData, int dwExtraInfo); [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);' -Name DISMISSER -Namespace System

$proc = Get-Process 'Diplomacy*' -ErrorAction SilentlyContinue
if (-not $proc) { Write-Host "Game not running"; exit 1 }
[DISMISSER]::SetForegroundWindow($proc.MainWindowHandle)
Start-Sleep -Milliseconds 800

# Press F10 to dismiss DINOForge panel if open
[DISMISSER]::keybd_event(0x79, 0, 0, 0); Start-Sleep -Milliseconds 100; [DISMISSER]::keybd_event(0x79, 0, 2, 0)
Start-Sleep -Milliseconds 500

# Also press Escape to close the sandbox dialog
[DISMISSER]::keybd_event(0x1B, 0, 0, 0); Start-Sleep -Milliseconds 100; [DISMISSER]::keybd_event(0x1B, 0, 2, 0)
Start-Sleep -Milliseconds 400

# Close any store popup at top-right
[DISMISSER]::SetCursorPos(1655, 15)
Start-Sleep -Milliseconds 200
[DISMISSER]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 100; [DISMISSER]::mouse_event(4, 0, 0, 0, 0)
Start-Sleep -Milliseconds 400

Write-Host "DINOForge panel dismissed, escape pressed"

# Now click on AUTOSAVE_1 row which is completely away from the DINOForge dialog area
# It's at y=340 (second save row)
[DISMISSER]::SetCursorPos(1000, 340)
Start-Sleep -Milliseconds 300
[DISMISSER]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 100; [DISMISSER]::mouse_event(4, 0, 0, 0, 0)
Start-Sleep -Milliseconds 800
Write-Host "Clicked AUTOSAVE_1 row"

# The save entry should now be selected - click the load/play button on its right side
[DISMISSER]::SetCursorPos(1360, 340)
Start-Sleep -Milliseconds 200
[DISMISSER]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 100; [DISMISSER]::mouse_event(4, 0, 0, 0, 0)
Write-Host "Clicked load button"
