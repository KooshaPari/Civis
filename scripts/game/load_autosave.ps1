Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd); [DllImport("user32.dll")] public static extern void mouse_event(int dwFlags, int dx, int dy, int dwData, int dwExtraInfo); [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y); [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo);' -Name LOADAUTO -Namespace System

$proc = Get-Process 'Diplomacy*' -ErrorAction SilentlyContinue
if (-not $proc) { Write-Host "Game not running"; exit 1 }
[LOADAUTO]::SetForegroundWindow($proc.MainWindowHandle)
Start-Sleep -Milliseconds 1000
Write-Host "Window focused"

# Press Escape to dismiss popup (top-right rotating popup)
[LOADAUTO]::keybd_event(0x1B, 0, 0, 0); Start-Sleep -Milliseconds 150; [LOADAUTO]::keybd_event(0x1B, 0, 2, 0)
Start-Sleep -Milliseconds 600
Write-Host "Pressed Escape"

# The load screen shows saves on the right panel
# AUTOSAVE_1 is at approximately y=340, centered at x=900-1000
# Let's click in the center of the AUTOSAVE_1 row
[LOADAUTO]::SetCursorPos(840, 340); Start-Sleep -Milliseconds 400
[LOADAUTO]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 150; [LOADAUTO]::mouse_event(4, 0, 0, 0, 0)
Start-Sleep -Milliseconds 600
Write-Host "Clicked AUTOSAVE_1 row at (840, 340)"

# After clicking the row, a LOAD button should appear on the right side
# Try the play/load icon at x=1360
[LOADAUTO]::SetCursorPos(1360, 340); Start-Sleep -Milliseconds 400
[LOADAUTO]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 150; [LOADAUTO]::mouse_event(4, 0, 0, 0, 0)
Write-Host "Clicked load button at (1360, 340)"
