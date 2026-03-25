Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd); [DllImport("user32.dll")] public static extern void mouse_event(int dwFlags, int dx, int dy, int dwData, int dwExtraInfo); [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);' -Name MENUCLICKER -Namespace System

$proc = Get-Process 'Diplomacy*' -ErrorAction SilentlyContinue
if (-not $proc) { Write-Host "Game not running"; exit 1 }
[MENUCLICKER]::SetForegroundWindow($proc.MainWindowHandle)
Start-Sleep -Milliseconds 800

# Close the WALLPAPERS popup at top-right (~x=1620, y=40 for close X button area)
[MENUCLICKER]::SetCursorPos(1620, 40); Start-Sleep -Milliseconds 300
[MENUCLICKER]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 100; [MENUCLICKER]::mouse_event(4, 0, 0, 0, 0)
Start-Sleep -Milliseconds 500
Write-Host "Closed wallpapers popup"

# Click CONTINUE from the left menu (appears to be around x=90, y=270 based on menu layout)
[MENUCLICKER]::SetCursorPos(90, 270); Start-Sleep -Milliseconds 300
[MENUCLICKER]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 100; [MENUCLICKER]::mouse_event(4, 0, 0, 0, 0)
Start-Sleep -Milliseconds 300
Write-Host "Clicked CONTINUE"
