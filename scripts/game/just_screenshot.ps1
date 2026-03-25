Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd); [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo);' -Name SCREENSHOTTER -Namespace System

$proc = Get-Process 'Diplomacy*' -ErrorAction SilentlyContinue
$hwnd = $proc.MainWindowHandle
[SCREENSHOTTER]::SetForegroundWindow($hwnd)
Start-Sleep -Milliseconds 600

# Press F10 to toggle DINOForge panel off
[SCREENSHOTTER]::keybd_event(0x79, 0, 0, 0); Start-Sleep -Milliseconds 100; [SCREENSHOTTER]::keybd_event(0x79, 0, 2, 0)
Start-Sleep -Milliseconds 300

# Also try pressing Escape to close the DINO game menus
[SCREENSHOTTER]::keybd_event(0x1B, 0, 0, 0); Start-Sleep -Milliseconds 100; [SCREENSHOTTER]::keybd_event(0x1B, 0, 2, 0)
Start-Sleep -Milliseconds 300

Write-Host "Done"
