Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd); [DllImport("user32.dll")] public static extern void mouse_event(int dwFlags, int dx, int dy, int dwData, int dwExtraInfo); [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);' -Name AUTOCLICKER -Namespace System

$proc = Get-Process 'Diplomacy*' -ErrorAction SilentlyContinue
$hwnd = $proc.MainWindowHandle
[AUTOCLICKER]::SetForegroundWindow($hwnd)
Start-Sleep -Milliseconds 600

# Close popup
[AUTOCLICKER]::SetCursorPos(1655, 15); Start-Sleep -Milliseconds 200
[AUTOCLICKER]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 100; [AUTOCLICKER]::mouse_event(4, 0, 0, 0, 0)
Start-Sleep -Milliseconds 500

# Take screenshot to confirm current state before clicking
Write-Host "Clicking AUTOSAVE_1 at y=340"
[AUTOCLICKER]::SetCursorPos(900, 340); Start-Sleep -Milliseconds 300
[AUTOCLICKER]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 100; [AUTOCLICKER]::mouse_event(4, 0, 0, 0, 0)
Start-Sleep -Milliseconds 1000

# After selecting, look for a "load" confirmation button - try bottom-center of the save panel
# Based on typical DINO UI, it might be at y=600 or so
Write-Host "Looking for load confirmation button"
# The save entry row itself shows expand - try clicking the rightmost icon (star or arrow) at x=1370
[AUTOCLICKER]::SetCursorPos(1370, 340); Start-Sleep -Milliseconds 200
[AUTOCLICKER]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 100; [AUTOCLICKER]::mouse_event(4, 0, 0, 0, 0)
Write-Host "Done"
