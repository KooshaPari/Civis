Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd); [DllImport("user32.dll")] public static extern void mouse_event(int dwFlags, int dx, int dy, int dwData, int dwExtraInfo); [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y); [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo);' -Name TRYCON -Namespace System

$proc = Get-Process 'Diplomacy*' -ErrorAction SilentlyContinue
if (-not $proc) { Write-Host "Game not running"; exit 1 }
[TRYCON]::SetForegroundWindow($proc.MainWindowHandle)
Start-Sleep -Milliseconds 1000

# Close popup with Escape first
[TRYCON]::keybd_event(0x1B, 0, 0, 0); Start-Sleep -Milliseconds 150; [TRYCON]::keybd_event(0x1B, 0, 2, 0)
Start-Sleep -Milliseconds 500

# Try clicking CONTINUE at several y positions (menu items appear around y=250-310)
# From the screenshot the menu is: GAY, CONTINUE, CHALLENGE MODE, SANDBOX MODE...
# "CONTINUE" seems to be the second item - try y=258
[TRYCON]::SetCursorPos(87, 258); Start-Sleep -Milliseconds 300
[TRYCON]::mouse_event(2, 0, 0, 0, 0); Start-Sleep -Milliseconds 150; [TRYCON]::mouse_event(4, 0, 0, 0, 0)
Start-Sleep -Milliseconds 300
Write-Host "Clicked at (87, 258)"
